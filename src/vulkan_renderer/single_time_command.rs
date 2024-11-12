use std::ops::Deref;

use ash::{prelude::VkResult, vk};

use crate::utils::{Defer, ScopeGuard};

use super::vulkan_interface::VulkanInterface;

pub struct SingleTimeCommand<'a> {
    command_buffer: vk::CommandBuffer,
    device: &'a ash::Device,
    interface: &'a VulkanInterface,
}

impl<'a> SingleTimeCommand<'a> {
    pub fn begin(device: &'a ash::Device, interface: &'a VulkanInterface) -> VkResult<Self> {
        let command_buffer = unsafe {
            device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_pool(interface.command_pool())
                    .command_buffer_count(1),
            )?[0]
                .defer(|command_buffer| {
                    device.free_command_buffers(interface.command_pool(), &[command_buffer])
                })
        };

        unsafe {
            device.begin_command_buffer(
                *command_buffer,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;
        }

        Ok(Self {
            command_buffer: ScopeGuard::into_inner(command_buffer),
            interface,
            device,
        })
    }

    pub fn submit(self) -> VkResult<()> {
        unsafe {
            self.device.end_command_buffer(self.command_buffer)?;

            self.device.queue_submit(
                self.interface.queues().graphics_queue(),
                &[vk::SubmitInfo::default().command_buffers(&[self.command_buffer])],
                vk::Fence::null(),
            )?;
            self.device
                .queue_wait_idle(self.interface.queues().graphics_queue())?;
        }
        Ok(())
    }
}

impl Deref for SingleTimeCommand<'_> {
    type Target = vk::CommandBuffer;

    fn deref(&self) -> &Self::Target {
        &self.command_buffer
    }
}

impl Drop for SingleTimeCommand<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .free_command_buffers(self.interface.command_pool(), &[self.command_buffer])
        }
    }
}
