mod builder;
mod queue;
mod sync_objects;

use crate::utils::{PipeLine, Result};
use ash::{prelude::VkResult, vk};
use builder::VulkanInterfaceBuilder;
use super::{render_targets::{self, RenderTargets}, vulkan_context::{QueueFamilies, VulkanContext}};
use queue::Queues;
pub use sync_objects::SyncObjects;
use crate::vulkan_renderer::NB_OF_FRAMES_IN_FLIGHT_USIZE;

pub struct VulkanInterface {
    queues: Queues,

    command_pool: vk::CommandPool,
    command_buffers: [vk::CommandBuffer; NB_OF_FRAMES_IN_FLIGHT_USIZE],

    sync_objects: SyncObjects,
}

impl VulkanInterface {
    pub unsafe fn new(context: &VulkanContext,
                      queue_families: QueueFamilies) -> Result<Self> {
        VulkanInterfaceBuilder::new(context)
            .create_queues(queue_families)
            .create_command_pool(queue_families)?
            .create_command_buffers()?
            .create_sync_objects()?
            .build()
            .pipe(Ok)
    }

    pub fn queues(&self) -> &Queues {
        &self.queues
    }

    pub fn command_buffers(&self) -> &[vk::CommandBuffer; NB_OF_FRAMES_IN_FLIGHT_USIZE] {
        &self.command_buffers
    }

    pub fn sync_objects(&self) -> &SyncObjects {
        &self.sync_objects
    }

    pub unsafe fn destroy(&mut self, context: &VulkanContext) {
        for semaphore in self.sync_objects.image_available_semaphores.into_iter() {
            unsafe { context.device().destroy_semaphore(semaphore, None) }
        }
        for semaphore in self.sync_objects.render_finished_semaphores.into_iter() {
            unsafe { context.device().destroy_semaphore(semaphore, None) }
        }
        for fence in self.sync_objects.in_flight_fences.into_iter() {
            unsafe { context.device().destroy_fence(fence, None) }
        }

        unsafe { context.device().destroy_command_pool(self.command_pool, None) }
    }
}

