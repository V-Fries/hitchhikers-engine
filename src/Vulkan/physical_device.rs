use super::errors::NoSuitablePhysicalDevice;

use ash::vk;
use anyhow::Result;

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct DeviceScore(u32);

struct ScoredDevice {
    device: vk::PhysicalDevice,
    score: DeviceScore,
}

#[derive(Default)]
struct QueueFamilies {
    graphics: Option<()>,
}

pub fn get_physical_device(instance: &ash::Instance) -> Result<vk::PhysicalDevice> {
    unsafe { instance.enumerate_physical_devices()? }
        .into_iter()
        .filter_map(|device| {
            filter_map_physical_device(instance, device)
        })
        .max_by(|left, right| { left.score.cmp(&right.score) })
        .map(|scores_device| scores_device.device)
        .ok_or(NoSuitablePhysicalDevice.into())
}

fn filter_map_physical_device(instance: &ash::Instance,
                              device: vk::PhysicalDevice) -> Option<ScoredDevice> {
    let device_properties = unsafe { instance.get_physical_device_properties(device) };
    let device_features = unsafe { instance.get_physical_device_features(device) };

    if !is_device_suitable(instance, device) {
        return None;
    }

    let score = score_device(device_properties, device_features);

    Some(ScoredDevice { device, score })
}

fn is_device_suitable(instance: &ash::Instance, device: vk::PhysicalDevice) -> bool {
    find_queue_families(instance, device)
        .graphics
        .is_some()
}

fn score_device(device_properties: vk::PhysicalDeviceProperties,
                _device_features: vk::PhysicalDeviceFeatures) -> DeviceScore {
    let mut score = DeviceScore(0);

    if device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        score.0 += 1000;
    }

    score
}

fn find_queue_families(instance: &ash::Instance,
                       device: vk::PhysicalDevice) -> QueueFamilies {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };
    let mut result = QueueFamilies::default();

    for queue_family in queue_families.into_iter() {
        if queue_family.queue_flags & vk::QueueFlags::GRAPHICS != vk::QueueFlags::default() {
            result.graphics = Some(());
        }
    }
    result
}
