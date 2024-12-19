mod from_texture_image;

use crate::vulkan_renderer::{
    buffer::Buffer, memory::Memory, single_time_command::SingleTimeCommand,
    vulkan_context::VulkanContext, vulkan_interface::VulkanInterface,
};
use ash::{prelude::VkResult, vk};
use rs42::{
    scope_guard::{Defer, ScopeGuard},
    Result,
};

pub struct Image {
    image: vk::Image,
    memory: vk::DeviceMemory,
    image_view: vk::ImageView,
    mip_levels: u32,
    #[cfg(debug_assertions)]
    is_destroyed: bool,
}

struct TransitionImageLayoutInfo {
    _format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_access_mask: vk::AccessFlags,
    dst_access_mask: vk::AccessFlags,
    src_stage_mask: vk::PipelineStageFlags,
    dst_stage_mask: vk::PipelineStageFlags,
}

pub struct ImageCreateInfo {
    pub mip_levels: u32,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
    pub properties: vk::MemoryPropertyFlags,
    pub aspect_mask: vk::ImageAspectFlags,
}

impl Image {
    pub fn new(context: &VulkanContext, image_create_info: ImageCreateInfo) -> Result<Self> {
        assert!(image_create_info.mip_levels >= 1);

        let image = Self::init_image(
            context.device(),
            image_create_info.extent,
            image_create_info.mip_levels,
            image_create_info.format,
            image_create_info.tiling,
            image_create_info.usage,
        )?
        .defer(|image| unsafe { context.device().destroy_image(image, None) });

        let memory = Self::init_memory(context, *image, image_create_info.properties)?
            .defer(|memory| unsafe { context.device().free_memory(memory, None) });

        unsafe { context.device().bind_image_memory(*image, *memory, 0)? };

        let image_view = unsafe {
            Self::init_image_view(
                context.device(),
                *image,
                image_create_info.format,
                image_create_info.aspect_mask,
                image_create_info.mip_levels,
            )?
        }
        .defer(|image_view| unsafe { context.device().destroy_image_view(image_view, None) });

        Ok(Self {
            image_view: ScopeGuard::into_inner(image_view),
            memory: ScopeGuard::into_inner(memory),
            image: ScopeGuard::into_inner(image),
            mip_levels: image_create_info.mip_levels,
            #[cfg(debug_assertions)]
            is_destroyed: false,
        })
    }

    fn init_image(
        device: &ash::Device,
        extent: vk::Extent2D,
        mip_levels: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
    ) -> VkResult<vk::Image> {
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(
                vk::Extent3D::default()
                    .width(extent.width)
                    .height(extent.height)
                    .depth(1),
            )
            .mip_levels(mip_levels)
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

    unsafe fn init_image_view(
        device: &ash::Device,
        image: vk::Image,
        format: vk::Format,
        aspect_mask: vk::ImageAspectFlags,
        mip_levels: u32,
    ) -> VkResult<vk::ImageView> {
        device.create_image_view(
            &vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(aspect_mask)
                        .base_mip_level(0)
                        .level_count(mip_levels)
                        .base_array_layer(0)
                        .layer_count(1),
                ),
            None,
        )
    }

    fn copy_from_buffer(
        &self,
        buffer: &Buffer,
        width: u32,
        height: u32,
        device: &ash::Device,
        interface: &VulkanInterface,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_destroyed)
        }

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

    pub fn image_view(&self) -> vk::ImageView {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_destroyed)
        }

        self.image_view
    }

    pub fn find_supported_format(
        context: &VulkanContext,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Option<vk::Format> {
        for format in candidates {
            let format_properties = unsafe {
                context
                    .instance()
                    .get_physical_device_format_properties(context.physical_device(), *format)
            };

            if tiling == vk::ImageTiling::LINEAR
                && (format_properties.linear_tiling_features & features) == features
            {
                return Some(*format);
            }
            if tiling == vk::ImageTiling::OPTIMAL
                && (format_properties.optimal_tiling_features & features) == features
            {
                return Some(*format);
            }
        }
        None
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_destroyed);
            self.is_destroyed = true;
        }

        device.destroy_image_view(self.image_view, None);
        device.destroy_image(self.image, None);
        device.free_memory(self.memory, None);
    }
}
