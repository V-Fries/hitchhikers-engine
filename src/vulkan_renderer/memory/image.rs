mod from_texture_image;
mod new;

use crate::vulkan_renderer::{
    buffer::Buffer, single_time_command::SingleTimeCommand, vulkan_context::VulkanContext,
    vulkan_interface::VulkanInterface,
};
use ash::vk;
pub use new::ImageCreateInfo;
use rs42::Result;

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

impl Image {
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
