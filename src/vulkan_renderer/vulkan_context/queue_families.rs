use ash::vk;
use crate::utils::{GetAllUniques, PipeLine};
use crate::vulkan_renderer::vulkan_context::errors::PhysicalDeviceIsNotSuitable;

#[derive(Clone, Copy)]
pub struct QueueFamilies {
    graphics_index: u32,
    present_index: u32,
}

#[derive(Default)]
pub struct QueueFamiliesBuilder {
    pub graphics_index: Option<usize>,
    pub present_index: Option<usize>,
}

impl QueueFamilies {
    pub fn graphics_index(&self) -> u32 {
        self.graphics_index
    }

    pub fn present_index(&self) -> u32 {
        self.present_index
    }

    pub fn as_vec_of_unique_indexes(&self) -> Vec<u32> {
        [self.graphics_index, self.present_index].into_iter().get_all_uniques()
    }
}

impl QueueFamiliesBuilder {
    pub fn build(&self, device: vk::PhysicalDevice) -> crate::utils::Result<QueueFamilies> {
        let option_to_u32 = |option: Option<usize>, queue_name: &str|
                             -> crate::utils::Result<u32, PhysicalDeviceIsNotSuitable> {
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
