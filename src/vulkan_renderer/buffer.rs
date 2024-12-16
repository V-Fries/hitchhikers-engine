use std::{ffi::c_void, ptr::copy_nonoverlapping};

use ash::vk;
use rs42::{
    defer,
    scope_guard::{Defer, ScopeGuard},
    Result,
};

use super::{
    memory::Memory, single_time_command::SingleTimeCommand, vulkan_context::VulkanContext,
    vulkan_interface::VulkanInterface,
};

pub struct Buffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    #[cfg(debug_assertions)]
    size: vk::DeviceSize,
    #[cfg(debug_assertions)]
    is_destroyed: bool,
}

impl Buffer {
    pub fn new(
        context: &VulkanContext,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        sharing_mode: vk::SharingMode,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        // TODO Should I assert the validity of the flags and sharing mode?

        unsafe {
            let buffer = context
                .device()
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(size)
                        .usage(usage)
                        .sharing_mode(sharing_mode),
                    None,
                )?
                .defer(|buffer| context.device().destroy_buffer(buffer, None));

            let memory = Self::init_memory(context, *buffer, properties)?
                .defer(|memory| context.device().free_memory(memory, None));

            context.device().bind_buffer_memory(*buffer, *memory, 0)?;

            Ok(Buffer {
                buffer: ScopeGuard::into_inner(buffer),
                memory: ScopeGuard::into_inner(memory),
                #[cfg(debug_assertions)]
                size,
                #[cfg(debug_assertions)]
                is_destroyed: false,
            })
        }
    }

    unsafe fn init_memory(
        context: &VulkanContext,
        buffer: vk::Buffer,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<vk::DeviceMemory> {
        unsafe {
            let memory_requirement = context.device().get_buffer_memory_requirements(buffer);

            let memory = context.device().allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(memory_requirement.size)
                    .memory_type_index(Memory::find_memory_type_index(
                        context,
                        memory_requirement.memory_type_bits,
                        properties,
                    )?),
                None,
            )?;

            Ok(memory)
        }
    }

    pub unsafe fn copy_from_ram<T>(
        &self,
        dst_offset: vk::DeviceSize,
        src: &[T],
        device: &ash::Device,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_destroyed);
            debug_assert!(self.size > dst_offset);
            debug_assert!(
                ((src.len() * size_of_val(&src[0])) as vk::DeviceSize) <= self.size - dst_offset
            );
        }

        let ptr = device.map_memory(
            self.memory,
            dst_offset,
            vk::WHOLE_SIZE,
            vk::MemoryMapFlags::empty(),
        )?;
        defer!(device.unmap_memory(self.memory));

        copy_nonoverlapping(
            src.as_ptr() as *const c_void,
            ptr,
            src.len() * size_of_val(&src[0]),
        );

        Ok(())
    }

    pub unsafe fn copy_from_buffer(
        &self,
        dst_offset: vk::DeviceSize,
        src: &Buffer,
        src_offset: vk::DeviceSize,
        size_to_copy: vk::DeviceSize,
        device: &ash::Device,
        interface: &VulkanInterface,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_destroyed);
            debug_assert!(self.size >= dst_offset);
            debug_assert!(src.size >= src_offset);
            debug_assert!(size_to_copy <= src.size - src_offset);
            debug_assert!(size_to_copy <= self.size - dst_offset);
        }

        let single_time_command = SingleTimeCommand::begin(device, interface)?;

        device.cmd_copy_buffer(
            *single_time_command,
            src.buffer,
            self.buffer,
            &[vk::BufferCopy::default()
                .src_offset(src_offset)
                .dst_offset(dst_offset)
                .size(size_to_copy)],
        );

        single_time_command.submit()?;

        Ok(())
    }

    pub fn buffer(&self) -> vk::Buffer {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_destroyed);
        }
        self.buffer
    }

    pub fn memory(&self) -> vk::DeviceMemory {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_destroyed);
        }
        self.memory
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_destroyed);
            self.is_destroyed = true;
        }
        device.destroy_buffer(self.buffer, None);
        device.free_memory(self.memory, None);
    }
}
