mod physical_device;
mod swapchain_builder;

use ash::vk;
use std::ffi::c_char;

pub use physical_device::PhysicalDeviceData;
use rs42::{extensions::PipeLine, Result};
pub use swapchain_builder::SwapchainBuilder;

pub const REQUIRED_EXTENSIONS: &[*const c_char] = &[
    #[cfg(target_os = "macos")]
    vk::KHR_PORTABILITY_SUBSET_NAME.as_ptr(),
    vk::KHR_SWAPCHAIN_NAME.as_ptr(),
];

pub unsafe fn create_device(
    instance: &ash::Instance,
    device_data: &PhysicalDeviceData,
) -> Result<ash::Device> {
    let queue_priority = [1.];
    let queue_create_infos: Vec<_> = device_data
        .queue_families
        .as_vec_of_unique_indexes()
        .into_iter()
        .map(|index| get_device_queue_create_info(index, &queue_priority))
        .collect();

    let device_features = get_device_features(&device_data.physical_device_features);
    let device_create_info = get_device_create_info(&queue_create_infos, &device_features);

    unsafe {
        instance
            .create_device(device_data.physical_device, &device_create_info, None)?
            .pipe(Ok)
    }
}

fn get_device_queue_create_info(
    queue_index: u32,
    queue_priority: &[f32],
) -> vk::DeviceQueueCreateInfo {
    vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_index)
        .queue_priorities(queue_priority)
}

fn get_device_features(
    physical_device_features: &vk::PhysicalDeviceFeatures,
) -> vk::PhysicalDeviceFeatures {
    vk::PhysicalDeviceFeatures::default()
        .sampler_anisotropy(physical_device_features.sampler_anisotropy != 0)
}

fn get_device_create_info<'a>(
    queue_create_infos: &'a [vk::DeviceQueueCreateInfo],
    device_features: &'a vk::PhysicalDeviceFeatures,
) -> vk::DeviceCreateInfo<'a> {
    vk::DeviceCreateInfo::default()
        .queue_create_infos(queue_create_infos)
        .enabled_features(device_features)
        .enabled_extension_names(REQUIRED_EXTENSIONS)
}
