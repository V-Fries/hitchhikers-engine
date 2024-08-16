use ash::vk;
use crate::utils::{PipeLine, Result};
use crate::vulkan_renderer::{NB_OF_FRAMES_IN_FLIGHT, NB_OF_FRAMES_IN_FLIGHT_USIZE};
use crate::vulkan_renderer::vulkan_interface::{Queues, SyncObjects, VulkanInterface};
use crate::vulkan_renderer::vulkan_context::{QueueFamilies, VulkanContext};

pub struct VulkanInterfaceBuilder<'a> {
    context: &'a VulkanContext,

    queues: Option<Queues>,

    command_pool: Option<vk::CommandPool>,
    command_buffers: Option<[vk::CommandBuffer; NB_OF_FRAMES_IN_FLIGHT_USIZE]>,

    sync_objects: Option<SyncObjects>,
}

impl<'a> VulkanInterfaceBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        Self {
            context,
            queues: None,
            command_pool: None,
            command_buffers: None,
            sync_objects: None,
        }
    }

    pub unsafe fn build(mut self) -> VulkanInterface {
        VulkanInterface {
            queues: self.queues.take().unwrap(),

            command_pool: self.command_pool.take().unwrap(),
            command_buffers: self.command_buffers.take().unwrap(),

            sync_objects: self.sync_objects.take().unwrap(),
        }
    }

    pub unsafe fn create_queues(mut self, queue_families: QueueFamilies) -> Self {
        self.queues = Some(Queues::new(self.context, queue_families));
        self
    }

    pub unsafe fn create_command_pool(mut self,
                                      queue_families: QueueFamilies)
                                      -> Result<Self> {
        let create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_families.graphics_index);

        self.command_pool = unsafe {
            self.context.device()
                .create_command_pool(&create_info, None)?
                .pipe(Some)
        };
        Ok(self)
    }

    pub unsafe fn create_command_buffers(mut self) -> Result<Self> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(NB_OF_FRAMES_IN_FLIGHT);

        self.command_buffers = Some(unsafe {
            self.context.device()
                .allocate_command_buffers(&alloc_info)?
                .as_slice()
                .try_into()?
        });
        Ok(self)
    }

    pub unsafe fn create_sync_objects(mut self)
                                      -> Result<Self> {
        self.sync_objects = SyncObjects::new(self.context.device(), NB_OF_FRAMES_IN_FLIGHT)?
            .pipe(Some);
        Ok(self)
    }

    unsafe fn command_pool(&self) -> vk::CommandPool {
        self.command_pool.unwrap()
    }
}

impl Drop for VulkanInterfaceBuilder<'_> {
    fn drop(&mut self) {
        self.destroy_sync_objects();
        self.destroy_command_pool();
    }
}

impl VulkanInterfaceBuilder<'_> {
    fn destroy_sync_objects(&mut self) {
        let Some(sync_objects) = self.sync_objects.take() else {
            return
        };

        for semaphore in sync_objects.image_available_semaphores.into_iter() {
            unsafe { self.context.device().destroy_semaphore(semaphore, None) }
        }
        for semaphore in sync_objects.render_finished_semaphores.into_iter() {
            unsafe { self.context.device().destroy_semaphore(semaphore, None) }
        }
        for fence in sync_objects.in_flight_fences.into_iter() {
            unsafe { self.context.device().destroy_fence(fence, None) }
        }
    }

    fn destroy_command_pool(&mut self) {
        if let Some(command_pool) = self.command_pool.take() {
            unsafe { self.context.device().destroy_command_pool(command_pool, None) }
        }
    }
}
