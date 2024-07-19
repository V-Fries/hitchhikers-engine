use ash::vk;
use crate::utils::{PipeLine, Result, TakeVec};

pub struct SyncObjects {
    pub image_available_semaphores: Box<[vk::Semaphore]>,
    pub render_finished_semaphores: Box<[vk::Semaphore]>,
    pub in_flight_fences: Box<[vk::Fence]>,
}

struct SyncObjectsBuilder<'a> {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    device: &'a ash::Device,
    nb_of_frames_in_flight: u32,
}

impl SyncObjects {
    pub fn new(device: &ash::Device, nb_of_frames_in_flight: u32) -> Result<Self> {
        SyncObjectsBuilder::new(device, nb_of_frames_in_flight)
            .image_available_semaphores()?
            .render_finished_semaphores()?
            .in_flight_fences()?
            .build()
            .pipe(Ok)
    }
}

impl<'a> SyncObjectsBuilder<'a> {
    fn new(device: &'a ash::Device, nb_of_frames_in_flight: u32) -> Self {
        let vec_size = nb_of_frames_in_flight as usize;
        Self {
            image_available_semaphores: Vec::with_capacity(vec_size),
            render_finished_semaphores: Vec::with_capacity(vec_size),
            in_flight_fences: Vec::with_capacity(vec_size),
            device,
            nb_of_frames_in_flight,
        }
    }

    fn image_available_semaphores(mut self) -> Result<Self> {
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        for _ in 0..self.nb_of_frames_in_flight {
            let semaphore = unsafe {
                self.device.create_semaphore(&semaphore_create_info, None)?
            };
            self.image_available_semaphores.push(semaphore);
        }
        Ok(self)
    }

    fn render_finished_semaphores(mut self) -> Result<Self> {
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        for _ in 0..self.nb_of_frames_in_flight {
            let semaphore = unsafe {
                self.device.create_semaphore(&semaphore_create_info, None)?
            };
            self.render_finished_semaphores.push(semaphore);
        }
        Ok(self)
    }

    fn in_flight_fences(mut self) -> Result<Self> {
        let fence_create_info = vk::FenceCreateInfo::default()
            .flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..self.nb_of_frames_in_flight {
            let fence = unsafe {
                self.device.create_fence(&fence_create_info, None)?
            };
            self.in_flight_fences.push(fence);
        }
        Ok(self)
    }

    fn build(mut self) -> SyncObjects {
        SyncObjects {
            image_available_semaphores: self.image_available_semaphores
                .take().into_boxed_slice(),
            render_finished_semaphores: self.render_finished_semaphores
                .take().into_boxed_slice(),
            in_flight_fences: self.in_flight_fences
                .take().into_boxed_slice(),
        }
    }
}

impl Drop for SyncObjectsBuilder<'_> {
    fn drop(&mut self) {
        destroy_semaphores(self.device, &self.image_available_semaphores);
        destroy_semaphores(self.device, &self.render_finished_semaphores);
        destroy_fences(self.device, &self.in_flight_fences);
    }
}

fn destroy_semaphores(device: &ash::Device, semaphores: &[vk::Semaphore]) {
    for semaphore in semaphores.iter() {
        unsafe { device.destroy_semaphore(*semaphore, None); }
    }
}

fn destroy_fences(device: &ash::Device, fences: &[vk::Fence]) {
    for fence in fences.iter() {
        unsafe { device.destroy_fence(*fence, None); }
    }
}
