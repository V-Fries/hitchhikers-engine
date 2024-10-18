use crate::{
    utils::Result,
    vulkan_renderer::{NB_OF_FRAMES_IN_FLIGHT, NB_OF_FRAMES_IN_FLIGHT_USIZE},
};
use ash::vk;

pub unsafe fn create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
) -> Result<[vk::CommandBuffer; NB_OF_FRAMES_IN_FLIGHT_USIZE]> {
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(NB_OF_FRAMES_IN_FLIGHT);

    Ok(device
        .allocate_command_buffers(&alloc_info)?
        .as_slice()
        .try_into()?)
}
