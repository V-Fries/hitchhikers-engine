mod builder;

use ash::{prelude::VkResult, vk};
use crate::{utils::{PipeLine, Result}, vulkan_renderer::vulkan_context::VulkanContext};
use builder::SyncObjectsBuilder;
use crate::vulkan_renderer::NB_OF_FRAMES_IN_FLIGHT_USIZE;

pub struct SyncObjects {
    pub image_available_semaphores: [vk::Semaphore; NB_OF_FRAMES_IN_FLIGHT_USIZE],
    pub render_finished_semaphores: [vk::Semaphore; NB_OF_FRAMES_IN_FLIGHT_USIZE],
    pub in_flight_fences: [vk::Fence; NB_OF_FRAMES_IN_FLIGHT_USIZE],
}

impl SyncObjects {
    pub unsafe fn new(device: &ash::Device, nb_of_frames_in_flight: u32) -> Result<Self> {
        SyncObjectsBuilder::new(device, nb_of_frames_in_flight)
            .create_image_available_semaphores(nb_of_frames_in_flight)?
            .create_render_finished_semaphores(nb_of_frames_in_flight)?
            .create_in_flight_fences(nb_of_frames_in_flight)?
            .build()?
            .pipe(Ok)
    }
}
