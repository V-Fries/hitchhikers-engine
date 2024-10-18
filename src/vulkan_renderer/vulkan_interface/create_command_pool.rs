use ash::{prelude::VkResult, vk};

pub unsafe fn create_command_pool(
    device: &ash::Device,
    graphics_queue_index: u32,
) -> VkResult<vk::CommandPool> {
    let create_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(graphics_queue_index);

    device.create_command_pool(&create_info, None)
}
