use ash::vk;
use crate::vulkan_renderer::vulkan_context::{QueueFamilies, VulkanContext};

pub struct Queues {
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

impl Queues {
    pub unsafe fn new(context: &VulkanContext, queue_families: QueueFamilies) -> Self {
        Queues {
            graphics_queue: context.device()
                .get_device_queue(queue_families.graphics_index, 0),
            present_queue: context.device()
                .get_device_queue(queue_families.present_index, 0),
        }
    }
}
