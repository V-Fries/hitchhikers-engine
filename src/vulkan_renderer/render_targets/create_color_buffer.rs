use ash::vk;

use crate::vulkan_renderer::{
    memory::{Image, ImageCreateInfo},
    vulkan_context::VulkanContext,
};
use rs42::Result;

pub fn create_color_buffer(
    context: &VulkanContext,
    swapchain_extent: vk::Extent2D,
    swapchain_image_format: vk::Format,
) -> Result<Image> {
    Image::new(
        context,
        ImageCreateInfo {
            mip_levels: 1,
            sample_count: context.physical_device_max_sample_count(),
            extent: swapchain_extent,
            format: swapchain_image_format,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::TRANSIENT_ATTACHMENT
                | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            aspect_mask: vk::ImageAspectFlags::COLOR,
        },
    )
}
