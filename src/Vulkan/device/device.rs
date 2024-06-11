use std::ffi::{c_char};
use ash::vk;

use super::physical_device::{get_physical_device, QueueFamilies};
use crate::utils::Result;

pub const REQUIRED_EXTENSIONS: &[*const c_char] = &[
    vk::KHR_PORTABILITY_SUBSET_NAME.as_ptr(),
];

pub fn create_device(instance: &ash::Instance) -> Result<ash::Device> {
    let (physical_device, queue_families) = get_physical_device(instance)?;

    let queue_priority = [1.];
    let graphics_queue_create_info = get_device_queue_create_info(queue_families,
                                                                  &queue_priority);
    let device_features = get_device_features();
    let queue_create_infos = [graphics_queue_create_info];
    let device_create_info = get_device_create_info(&queue_create_infos,
                                                    &device_features);

    Ok(unsafe { instance.create_device(physical_device, &device_create_info, None)? })
}

fn get_device_queue_create_info(queue_families: QueueFamilies,
                                queue_priority: &[f32])
                                -> vk::DeviceQueueCreateInfo {
    vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_families.graphics_index)
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
