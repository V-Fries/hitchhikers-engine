use std::ffi::{c_char};
use ash::vk;

use super::physical_device::{get_physical_device};
use crate::utils::Result;
use crate::vulkan::swap_chain::SwapChainBuilder;

pub const REQUIRED_EXTENSIONS: &[*const c_char] = &[
    vk::KHR_PORTABILITY_SUBSET_NAME.as_ptr(),
    vk::KHR_SWAPCHAIN_NAME.as_ptr(),
];

pub struct Queues {
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

pub unsafe fn create_device(entry: &ash::Entry,
                            instance: &ash::Instance,
                            surface: vk::SurfaceKHR,
                            window_inner_size: winit::dpi::PhysicalSize<u32>)
                            -> Result<(ash::Device, Queues, SwapChainBuilder)> {
    let device_data = get_physical_device(
        entry, instance, surface, window_inner_size,
    )?;

    let queue_priority = [1.];
    let queue_create_infos: Vec<_> = device_data.queue_families
        .as_vec_of_unique_indexes()
        .into_iter()
        .map(|index| {
            get_device_queue_create_info(index, &queue_priority)
        })
        .collect();

    let device_features = get_device_features();
    let device_create_info = get_device_create_info(&queue_create_infos,
                                                    &device_features);

    let device = instance.
        create_device(device_data.physical_device, &device_create_info, None)?;
    let graphics_queue = device
        .get_device_queue(device_data.queue_families.graphics_index, 0);
    let present_queue = device
        .get_device_queue(device_data.queue_families.graphics_index, 0);
    let queues = Queues { graphics_queue, present_queue };
    Ok((device, queues, device_data.swap_chain_builder))
}

fn get_device_queue_create_info(queue_index: u32,
                                queue_priority: &[f32])
                                -> vk::DeviceQueueCreateInfo {
    vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_index)
        .queue_priorities(queue_priority)
}

fn get_device_features() -> vk::PhysicalDeviceFeatures {
    vk::PhysicalDeviceFeatures::default()
}

fn get_device_create_info<'a>(queue_create_infos: &'a [vk::DeviceQueueCreateInfo],
                              device_features: &'a vk::PhysicalDeviceFeatures)
                              -> vk::DeviceCreateInfo<'a> {
    // May need to add validation layers for old versions of vulkan
    vk::DeviceCreateInfo::default()
        .queue_create_infos(queue_create_infos)
        .enabled_features(device_features)
        .enabled_extension_names(REQUIRED_EXTENSIONS)
}
