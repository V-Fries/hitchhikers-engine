mod builder;
mod queue;
mod sync_objects;

use super::vulkan_context::{QueueFamilies, VulkanContext};
use crate::utils::{PipeLine, Result};
use crate::vulkan_renderer::NB_OF_FRAMES_IN_FLIGHT_USIZE;
use ash::vk;
use builder::VulkanInterfaceBuilder;
use queue::Queues;
pub use sync_objects::SyncObjects;

pub struct VulkanInterface {
    is_destroyed: bool,

    queues: Queues,
    queue_families: QueueFamilies, // TODO check if I really need to store this and update excalidraw

    command_pool: vk::CommandPool,
    command_buffers: [vk::CommandBuffer; NB_OF_FRAMES_IN_FLIGHT_USIZE],

    sync_objects: SyncObjects,
}

impl VulkanInterface {
    pub unsafe fn new(context: &VulkanContext, queue_families: QueueFamilies) -> Result<Self> {
        VulkanInterfaceBuilder::new(context, queue_families)
            .create_queues()
            .create_command_pool()?
            .create_command_buffers()?
            .create_sync_objects()?
            .build()
            .pipe(Ok)
    }

    pub fn queues(&self) -> &Queues {
        #[cfg(feature = "validation_layers")]
        {
            assert!(
                !self.is_destroyed,
                "VulkanInterface::queues() was called after interface destruction"
            );
        }
        &self.queues
    }

    pub fn queue_families(&self) -> QueueFamilies {
        #[cfg(feature = "validation_layers")]
        {
            assert!(
                !self.is_destroyed,
                "VulkanInterface::queue_families() was called after interface destruction"
            );
        }
        self.queue_families
    }

    pub fn command_pool(&self) -> vk::CommandPool {
        #[cfg(feature = "validation_layers")]
        {
            assert!(
                !self.is_destroyed,
                "VulkanInterface::command_pool() was called after interface destruction"
            );
        }
        self.command_pool
    }

    pub fn command_buffers(&self) -> &[vk::CommandBuffer; NB_OF_FRAMES_IN_FLIGHT_USIZE] {
        #[cfg(feature = "validation_layers")]
        {
            assert!(
                !self.is_destroyed,
                "VulkanInterface::command_buffers() was called after interface destruction"
            );
        }
        &self.command_buffers
    }

    pub fn sync_objects(&self) -> &SyncObjects {
        #[cfg(feature = "validation_layers")]
        {
            assert!(
                !self.is_destroyed,
                "VulkanInterface::sync_objects() was called after interface destruction"
            );
        }
        &self.sync_objects
    }

    pub unsafe fn destroy(&mut self, context: &VulkanContext) {
        if self.is_destroyed {
            return;
        }
        self.is_destroyed = true;

        for semaphore in self.sync_objects.image_available_semaphores.into_iter() {
            unsafe { context.device().destroy_semaphore(semaphore, None) }
        }
        for semaphore in self.sync_objects.render_finished_semaphores.into_iter() {
            unsafe { context.device().destroy_semaphore(semaphore, None) }
        }
        for fence in self.sync_objects.in_flight_fences.into_iter() {
            unsafe { context.device().destroy_fence(fence, None) }
        }

        unsafe {
            context
                .device()
                .destroy_command_pool(self.command_pool, None)
        }
    }
}
