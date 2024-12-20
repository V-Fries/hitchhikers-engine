use ash::{
    prelude::VkResult,
    vk::{self, Offset3D},
};

use rs42::{
    extensions::PipeLine,
    scope_guard::{Defer, ScopeGuard},
    Result,
};

use crate::vulkan_renderer::{
    buffer::Buffer, single_time_command::SingleTimeCommand, vulkan_context::VulkanContext,
    vulkan_interface::VulkanInterface,
};

use super::{Image, ImageCreateInfo, TransitionImageLayoutInfo};

impl Image {
    pub unsafe fn from_texture_image(
        context: &VulkanContext,
        interface: &VulkanInterface,
        texture: &image_parser::Image,
    ) -> Result<Self> {
        let image_format = vk::Format::R8G8B8A8_SRGB;
        let mip_levels = get_mip_level(context, texture, image_format);

        let staging_buffer = create_staging_buffer(context, texture)?
            .defer(|mut staging_buffer| staging_buffer.destroy(context.device()));

        let image = create_image(context, texture, mip_levels, image_format)?
            .defer(|mut image| image.destroy(context.device()));

        copy_staging_buffer_to_image_and_generate_mip_maps(
            &image,
            &staging_buffer,
            context.device(),
            interface,
            texture,
        )?;

        ScopeGuard::into_inner(image).pipe(Ok)
    }
}

fn create_staging_buffer(context: &VulkanContext, texture: &image_parser::Image) -> Result<Buffer> {
    let staging_buffer = Buffer::new(
        context,
        (texture.width() * texture.height() * size_of_val(&texture[0])) as vk::DeviceSize,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::SharingMode::EXCLUSIVE,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?
    .defer(|mut staging_buffer| unsafe { staging_buffer.destroy(context.device()) });

    unsafe { staging_buffer.copy_from_ram(0, texture, context.device())? }

    ScopeGuard::into_inner(staging_buffer).pipe(Ok)
}

unsafe fn get_mip_level(
    context: &VulkanContext,
    texture: &image_parser::Image,
    image_format: vk::Format,
) -> u32 {
    let format_properties = context
        .instance()
        .get_physical_device_format_properties(context.physical_device(), image_format);

    if format_properties.optimal_tiling_features
        & vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR
        == vk::FormatFeatureFlags::empty()
    {
        return 1;
    }

    (texture.width() as f32)
        .max(texture.height() as f32)
        .log2()
        .floor() as u32
        + 1
}

fn create_image(
    context: &VulkanContext,
    texture: &image_parser::Image,
    mip_levels: u32,
    image_format: vk::Format,
) -> Result<Image> {
    Image::new(
        context,
        ImageCreateInfo {
            mip_levels,
            sample_count: vk::SampleCountFlags::TYPE_1,
            extent: vk::Extent2D {
                width: texture.width() as u32,
                height: texture.height() as u32,
            },
            format: image_format,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            aspect_mask: vk::ImageAspectFlags::COLOR,
        },
    )
}

unsafe fn copy_staging_buffer_to_image_and_generate_mip_maps(
    image: &Image,
    staging_buffer: &Buffer,
    device: &ash::Device,
    interface: &VulkanInterface,
    texture: &image_parser::Image,
) -> Result<()> {
    // TODO the next 3 function call all create a SingleTimeCommand, make them share a single
    // command buffer
    // Might be worth looking into creating a single SingleTimeCommand per frame

    transition_image_layout_from_undefined_to_transfer_dst_optimal(
        image,
        device,
        interface,
        vk::Format::R8G8B8A8_SRGB,
    )?;

    image.copy_from_buffer(
        staging_buffer,
        texture.width() as u32,
        texture.height() as u32,
        device,
        interface,
    )?;

    generate_mip_maps(
        image,
        vk::Extent2D {
            width: texture.width() as u32,
            height: texture.height() as u32,
        },
        device,
        interface,
    )?;

    Ok(())
}

fn transition_image_layout_from_undefined_to_transfer_dst_optimal(
    image: &Image,
    device: &ash::Device,
    interface: &VulkanInterface,
    format: vk::Format,
) -> VkResult<()> {
    unsafe {
        transition_image_layout(
            image,
            device,
            interface,
            TransitionImageLayoutInfo {
                _format: format,
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                src_stage_mask: vk::PipelineStageFlags::TOP_OF_PIPE,
                dst_stage_mask: vk::PipelineStageFlags::TRANSFER,
            },
        )
    }
}

unsafe fn transition_image_layout(
    image: &Image,
    device: &ash::Device,
    interface: &VulkanInterface,
    info: TransitionImageLayoutInfo,
) -> VkResult<()> {
    let single_time_command = SingleTimeCommand::begin(device, interface)?;

    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(info.old_layout)
        .new_layout(info.new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image.image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(image.mip_levels)
                .base_array_layer(0)
                .layer_count(1),
        )
        .src_access_mask(info.src_access_mask)
        .dst_access_mask(info.dst_access_mask);

    device.cmd_pipeline_barrier(
        *single_time_command,
        info.src_stage_mask,
        info.dst_stage_mask,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
    );

    single_time_command.submit()?;
    Ok(())
}

unsafe fn generate_mip_maps(
    image: &Image,
    extent: vk::Extent2D,
    device: &ash::Device,
    interface: &VulkanInterface,
) -> VkResult<()> {
    let single_time_command = SingleTimeCommand::begin(device, interface)?;
    let mut mip_width = extent.width as i32;
    let mut mip_height = extent.height as i32;

    let mut barrier = [vk::ImageMemoryBarrier::default()
        .image(image.image)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_array_layer(0)
                .layer_count(1)
                .level_count(1),
        )];

    for i in 1..image.mip_levels {
        barrier[0].subresource_range.base_mip_level = i - 1;
        barrier[0].old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier[0].new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier[0].src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier[0].dst_access_mask = vk::AccessFlags::TRANSFER_READ;

        device.cmd_pipeline_barrier(
            *single_time_command,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &barrier,
        );

        let blit = [vk::ImageBlit::default()
            .src_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: mip_width,
                    y: mip_height,
                    z: 1,
                },
            ])
            .src_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(i - 1)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .dst_offsets([
                Offset3D { x: 0, y: 0, z: 0 },
                Offset3D {
                    x: (mip_width / 2).max(1),
                    y: (mip_height / 2).max(1),
                    z: 1,
                },
            ])
            .dst_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(i)
                    .base_array_layer(0)
                    .layer_count(1),
            )];

        device.cmd_blit_image(
            *single_time_command,
            image.image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            image.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &blit,
            vk::Filter::LINEAR,
        );

        barrier[0].old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier[0].new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier[0].src_access_mask = vk::AccessFlags::TRANSFER_READ;
        barrier[0].dst_access_mask = vk::AccessFlags::SHADER_READ;

        device.cmd_pipeline_barrier(
            *single_time_command,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &barrier,
        );

        if mip_width > 1 {
            mip_width /= 2;
        }
        if mip_height > 1 {
            mip_height /= 2;
        }
    }

    barrier[0].subresource_range.base_mip_level = image.mip_levels - 1;
    barrier[0].old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    barrier[0].new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    barrier[0].src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    barrier[0].dst_access_mask = vk::AccessFlags::SHADER_READ;

    device.cmd_pipeline_barrier(
        *single_time_command,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &barrier,
    );

    single_time_command.submit()?;
    Ok(())
}
