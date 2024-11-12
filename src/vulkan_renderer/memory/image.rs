use crate::{
    utils::{Defer, Result, ScopeGuard},
    vulkan_renderer::{
        buffer::Buffer, memory::Memory, single_time_command::SingleTimeCommand,
        vulkan_context::VulkanContext, vulkan_interface::VulkanInterface,
    },
};
use ash::{prelude::VkResult, vk};

pub struct Image {
    image: vk::Image,
    memory: vk::DeviceMemory,
    #[cfg(debug_assertions)]
    is_destroyed: bool,
}

impl Image {
    pub fn new(
        context: &VulkanContext,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        let image = Self::init_image(context.device(), width, height, format, tiling, usage)?
            .defer(|image| unsafe { context.device().destroy_image(image, None) });

        let memory = Self::init_memory(context, *image, properties)?
            .defer(|memory| unsafe { context.device().free_memory(memory, None) });

        unsafe { context.device().bind_image_memory(*image, *memory, 0)? };

        Ok(Self {
            memory: ScopeGuard::into_inner(memory),
            image: ScopeGuard::into_inner(image),
            #[cfg(debug_assertions)]
            is_destroyed: false,
        })
    }

    fn init_image(
        device: &ash::Device,
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
    ) -> VkResult<vk::Image> {
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D::default().width(width).height(height).depth(1))
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        // TODO handle the case where sRGB is not supported
        unsafe { device.create_image(&image_create_info, None) }
    }

    fn init_memory(
        context: &VulkanContext,
        image: vk::Image,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<vk::DeviceMemory> {
        let mem_requirements = unsafe { context.device().get_image_memory_requirements(image) };
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Memory::find_memory_type_index(
                context,
                mem_requirements.memory_type_bits,
                properties,
            )?);
        Ok(unsafe { context.device().allocate_memory(&alloc_info, None)? })
    }

    pub fn from_texture_image(
        context: &VulkanContext,
        interface: &VulkanInterface,
        texture: &image_parser::Image,
    ) -> Result<Self> {
        // TODO Refactor
        let staging_buffer = Buffer::new(
            context,
            (texture.width() * texture.height() * size_of_val(&texture[0])) as vk::DeviceSize,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::SharingMode::EXCLUSIVE,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?
        .defer(|mut staging_buffer| unsafe { staging_buffer.destroy(context.device()) });

        unsafe { staging_buffer.copy_from_ram(0, texture, context.device())? };

        let image = Self::new(
            context,
            texture.width() as u32,
            texture.height() as u32,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?
        .defer(|mut image| unsafe { image.destroy(context.device()) });

        // TODO the next 3 function call all create a SingleTimeCommand, make them share a single
        // command buffer
        // Might be worth looking into creating a single SingleTimeCommand per frame

        image.transition_image_layout_from_undefined_to_transfer_dst_optimal(
            context.device(),
            interface,
            vk::Format::R8G8B8A8_SRGB,
        )?;

        image.copy_from_buffer(
            &staging_buffer,
            texture.width() as u32,
            texture.height() as u32,
            context.device(),
            interface,
        )?;

        image.transition_image_layout_from_transfer_dst_optimal_to_shader_read_only_optimal(
            context.device(),
            interface,
            vk::Format::R8G8B8A8_SRGB,
        )?;

        Ok(ScopeGuard::into_inner(image))
    }

    fn transition_image_layout_from_undefined_to_transfer_dst_optimal(
        &self,
        device: &ash::Device,
        interface: &VulkanInterface,
        format: vk::Format,
    ) -> VkResult<()> {
        unsafe {
            self.transition_image_layout(
                device,
                interface,
                format,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            )
        }
    }

    fn transition_image_layout_from_transfer_dst_optimal_to_shader_read_only_optimal(
        &self,
        device: &ash::Device,
        interface: &VulkanInterface,
        format: vk::Format,
    ) -> VkResult<()> {
        unsafe {
            self.transition_image_layout(
                device,
                interface,
                format,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            )
        }
    }

    unsafe fn transition_image_layout(
        &self,
        device: &ash::Device,
        interface: &VulkanInterface,
        _format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
    ) -> VkResult<()> {
        let single_time_command = SingleTimeCommand::begin(device, interface)?;

        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.image)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask);

        device.cmd_pipeline_barrier(
            *single_time_command,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );

        single_time_command.submit()?;
        Ok(())
    }

    fn copy_from_buffer(
        &self,
        buffer: &Buffer,
        width: u32,
        height: u32,
        device: &ash::Device,
        interface: &VulkanInterface,
    ) -> Result<()> {
        let single_time_command = SingleTimeCommand::begin(device, interface)?;

        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .image_offset(vk::Offset3D::default().x(0).y(0).z(0))
            .image_extent(vk::Extent3D::default().width(width).height(height).depth(1));

        unsafe {
            device.cmd_copy_buffer_to_image(
                *single_time_command,
                buffer.buffer(),
                self.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        single_time_command.submit()?;
        Ok(())
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_destroyed);
            self.is_destroyed = true;
        }

        device.destroy_image(self.image, None);
        device.free_memory(self.memory, None);
    }
}
