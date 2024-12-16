use crate::vulkan_renderer::vulkan_context::errors::PhysicalDeviceIsNotSuitable;
use ash::vk;
use rs42::{extensions::{iterator::GetAllUniques, PipeLine}, Result};

#[derive(Clone, Copy)]
pub struct QueueFamilies {
    pub graphics_index: u32,
    pub present_index: u32,
}

#[derive(Default)]
pub struct QueueFamiliesBuilder {
    pub graphics_index: Option<usize>,
    pub present_index: Option<usize>,
}

impl QueueFamilies {
    pub fn as_vec_of_unique_indexes(&self) -> Vec<u32> {
        [self.graphics_index, self.present_index]
            .into_iter()
            .get_all_uniques()
    }
}

impl QueueFamiliesBuilder {
    pub fn build(&self, device: vk::PhysicalDevice) -> Result<QueueFamilies> {
        let option_to_u32 =
            |option: Option<usize>, queue_name: &str| -> Result<u32, PhysicalDeviceIsNotSuitable> {
                option
                    .ok_or(PhysicalDeviceIsNotSuitable::new(
                        device,
                        format!("{queue_name} queue is not supported"),
                    ))?
                    .pipe(|p| p as u32)
                    .pipe(Ok)
            };

        Ok(QueueFamilies {
            graphics_index: option_to_u32(self.graphics_index, "graphics")?,
            present_index: option_to_u32(self.present_index, "present")?,
        })
    }
}
