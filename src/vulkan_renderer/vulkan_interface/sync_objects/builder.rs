use super::SyncObjects;
use crate::utils::{Result, TakeVec};
use ash::vk;
use std::convert::TryInto;

pub struct SyncObjectsBuilder<'a> {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    device: &'a ash::Device,
}

impl<'a> SyncObjectsBuilder<'a> {
    pub fn new(device: &'a ash::Device, nb_of_frames_in_flight: u32) -> Self {
        let vec_size = nb_of_frames_in_flight as usize;
        Self {
            image_available_semaphores: Vec::with_capacity(vec_size),
            render_finished_semaphores: Vec::with_capacity(vec_size),
            in_flight_fences: Vec::with_capacity(vec_size),
            device,
        }
    }

    pub unsafe fn build(mut self) -> Result<SyncObjects> {
        Ok(SyncObjects {
            image_available_semaphores: self
                .image_available_semaphores
                .take()
                .as_slice()
                .try_into()
                .expect("image_available_semaphores was not initialized"),
            render_finished_semaphores: self
                .render_finished_semaphores
                .take()
                .as_slice()
                .try_into()
                .expect("render_finished_semaphores was not initialized"),
            in_flight_fences: self
                .in_flight_fences
                .take()
                .as_slice()
                .try_into()
                .expect("in_flight_fences was not initialized"),
        })
    }

    pub unsafe fn create_image_available_semaphores(
        mut self,
        nb_of_frames_in_flight: u32,
    ) -> Result<Self> {
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        for _ in 0..nb_of_frames_in_flight {
            let semaphore = unsafe { self.device.create_semaphore(&semaphore_create_info, None)? };
            self.image_available_semaphores.push(semaphore);
        }
        Ok(self)
    }

    pub unsafe fn create_render_finished_semaphores(
        mut self,
        nb_of_frames_in_flight: u32,
    ) -> Result<Self> {
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        for _ in 0..nb_of_frames_in_flight {
            let semaphore = unsafe { self.device.create_semaphore(&semaphore_create_info, None)? };
            self.render_finished_semaphores.push(semaphore);
        }
        Ok(self)
    }

    pub unsafe fn create_in_flight_fences(mut self, nb_of_frames_in_flight: u32) -> Result<Self> {
        let fence_create_info =
            vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..nb_of_frames_in_flight {
            let fence = unsafe { self.device.create_fence(&fence_create_info, None)? };
            self.in_flight_fences.push(fence);
        }
        Ok(self)
    }
}

impl Drop for SyncObjectsBuilder<'_> {
    fn drop(&mut self) {
        Self::destroy_semaphores(self.device, &self.image_available_semaphores);
        Self::destroy_semaphores(self.device, &self.render_finished_semaphores);
        Self::destroy_fences(self.device, &self.in_flight_fences);
    }
}

impl SyncObjectsBuilder<'_> {
    fn destroy_semaphores(device: &ash::Device, semaphores: &[vk::Semaphore]) {
        for semaphore in semaphores.iter() {
            unsafe {
                device.destroy_semaphore(*semaphore, None);
            }
        }
    }

    fn destroy_fences(device: &ash::Device, fences: &[vk::Fence]) {
        for fence in fences.iter() {
            unsafe {
                device.destroy_fence(*fence, None);
            }
        }
    }
}
