use super::errors::NoSuitablePhysicalDevice;

use ash::vk;
use anyhow::Result;

type DeviceScore = u32;

struct ScoredDevice {
    device: vk::PhysicalDevice,
    score: DeviceScore,
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

    if !is_device_suitable(device_properties, device_features) {
        return None;
    }

    let score = score_device(device_properties, device_features);

    Some(ScoredDevice { device, score })
}

fn is_device_suitable(_device_properties: vk::PhysicalDeviceProperties,
                      _device_features: vk::PhysicalDeviceFeatures) -> bool {
    // TODO evaluate device suitability (see example below)
    // if device_features.geometry_shader == 0 {
    //     return false;
    // }
    true
}

fn score_device(device_properties: vk::PhysicalDeviceProperties,
                _device_features: vk::PhysicalDeviceFeatures) -> DeviceScore {
    let mut score: DeviceScore = 0;

    if device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        score += 1000;
    }

    score
}
