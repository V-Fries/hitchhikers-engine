use ash::vk;

use crate::{
    utils::Result,
    vulkan_renderer::{memory::Image, vulkan_context::VulkanContext},
};

use super::errors::FailedToFindSupportedFormatForDepthBuffer;

pub fn create_depth_buffer(
    context: &VulkanContext,
    swapchain_extent: vk::Extent2D,
) -> Result<Image> {
    Image::new(
        context,
        swapchain_extent,
        find_depth_buffer_format(context)?,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::ImageAspectFlags::DEPTH,
    )
}

pub fn find_depth_buffer_format(
    context: &VulkanContext,
) -> Result<vk::Format, FailedToFindSupportedFormatForDepthBuffer> {
    Image::find_supported_format(
        context,
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
    .ok_or(FailedToFindSupportedFormatForDepthBuffer {})
}
