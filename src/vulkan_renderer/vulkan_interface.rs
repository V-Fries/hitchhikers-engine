mod create_command_buffers;
mod create_command_pool;
mod queue;
mod sync_objects;

use super::vulkan_context::{QueueFamilies, VulkanContext};
use super::NB_OF_FRAMES_IN_FLIGHT;
use crate::utils::{Defer, Result, ScopeGuard};
use crate::vulkan_renderer::NB_OF_FRAMES_IN_FLIGHT_USIZE;
use ash::vk;
use create_command_buffers::create_command_buffers;
use create_command_pool::create_command_pool;
use queue::Queues;
pub use sync_objects::SyncObjects;

pub struct VulkanInterface {
    is_destroyed: bool,

    queues: Queues,
    queue_families: QueueFamilies,

    command_pool: vk::CommandPool,
    command_buffers: [vk::CommandBuffer; NB_OF_FRAMES_IN_FLIGHT_USIZE],

    sync_objects: SyncObjects,
}

impl VulkanInterface {
    pub unsafe fn new(context: &VulkanContext, queue_families: QueueFamilies) -> Result<Self> {
        let queues = Queues::new(context, queue_families);
        let command_pool = create_command_pool(context.device(), queue_families.graphics_index)?
            .defer(|command_pool| context.device().destroy_command_pool(command_pool, None));
        let command_buffers = create_command_buffers(context.device(), *command_pool)?;
        let sync_objects = SyncObjects::new(context.device(), NB_OF_FRAMES_IN_FLIGHT)?
            .defer(|sync_objects| sync_objects.destroy(context.device()));

        Ok(VulkanInterface {
            sync_objects: ScopeGuard::into_inner(sync_objects),
            command_buffers,
            command_pool: ScopeGuard::into_inner(command_pool),
            queue_families,
            queues,
            is_destroyed: false,
        })
    }

    pub fn queues(&self) -> &Queues {
        debug_assert!(
            !self.is_destroyed,
            "VulkanInterface::queues() was called after interface destruction"
        );
        &self.queues
    }

    pub fn queue_families(&self) -> QueueFamilies {
        debug_assert!(
            !self.is_destroyed,
            "VulkanInterface::queue_families() was called after interface destruction"
        );
        self.queue_families
    }

    pub fn command_pool(&self) -> vk::CommandPool {
        debug_assert!(
            !self.is_destroyed,
            "VulkanInterface::command_pool() was called after interface destruction"
        );
        self.command_pool
    }

    pub fn command_buffers(&self) -> &[vk::CommandBuffer; NB_OF_FRAMES_IN_FLIGHT_USIZE] {
        debug_assert!(
            !self.is_destroyed,
            "VulkanInterface::command_buffers() was called after interface destruction"
        );
        &self.command_buffers
    }

    pub fn sync_objects(&self) -> &SyncObjects {
        debug_assert!(
            !self.is_destroyed,
            "VulkanInterface::sync_objects() was called after interface destruction"
        );
        &self.sync_objects
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        // If an error occurs during swapchain recreating this function might be called twice
        if self.is_destroyed {
            return;
        }
        self.is_destroyed = true;

        self.sync_objects.destroy(device);

        device.destroy_command_pool(self.command_pool, None);
    }
}
