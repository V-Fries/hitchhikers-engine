use crate::vulkan_renderer::vulkan_context::{QueueFamilies, VulkanContext};
use ash::vk;

pub struct Queues {
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

impl Queues {
    pub unsafe fn new(context: &VulkanContext, queue_families: QueueFamilies) -> Self {
        Queues {
            graphics_queue: context
                .device()
                .get_device_queue(queue_families.graphics_index, 0),
            present_queue: context
                .device()
                .get_device_queue(queue_families.present_index, 0),
        }
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }
}
