use ash::vk;

use crate::vulkan_renderer::{
    memory::{Image, ImageCreateInfo},
    vulkan_context::VulkanContext,
};
use rs42::Result;

use super::errors::FailedToFindSupportedFormatForDepthBuffer;

pub fn create_depth_buffer(
    context: &VulkanContext,
    swapchain_extent: vk::Extent2D,
) -> Result<Image> {
    Image::new(
        context,
        ImageCreateInfo {
            mip_levels: 1,
            sample_count: context.physical_device_max_sample_count(),
            extent: swapchain_extent,
            format: find_depth_buffer_format(context)?,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            aspect_mask: vk::ImageAspectFlags::DEPTH,
        },
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
