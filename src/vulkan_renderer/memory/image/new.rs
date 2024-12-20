use crate::vulkan_renderer::{memory::Memory, vulkan_context::VulkanContext};
use ash::{prelude::VkResult, vk};
use rs42::{
    scope_guard::{Defer, ScopeGuard},
    Result,
};

use super::Image;

pub struct ImageCreateInfo {
    pub mip_levels: u32,
    pub sample_count: vk::SampleCountFlags,
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
        if image_create_info.mip_levels != 1 {
            assert_eq!(image_create_info.sample_count, vk::SampleCountFlags::TYPE_1);
        }

        let image = init_image(
            context.device(),
            image_create_info.extent,
            image_create_info.mip_levels,
            image_create_info.sample_count,
            image_create_info.format,
            image_create_info.tiling,
            image_create_info.usage,
        )?
        .defer(|image| unsafe { context.device().destroy_image(image, None) });

        let memory = unsafe { init_memory(context, *image, image_create_info.properties)? }
            .defer(|memory| unsafe { context.device().free_memory(memory, None) });

        unsafe { context.device().bind_image_memory(*image, *memory, 0)? };

        let image_view = unsafe {
            init_image_view(
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
}

fn init_image(
    device: &ash::Device,
    extent: vk::Extent2D,
    mip_levels: u32,
    sample_count: vk::SampleCountFlags,
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
        .samples(sample_count)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    // TODO handle the case where sRGB is not supported
    unsafe { device.create_image(&image_create_info, None) }
}

unsafe fn init_memory(
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
