use std::collections::HashSet;
use std::ffi::CStr;
use crate::vulkan::errors::{NoSuitablePhysicalDevice, PhysicalDeviceIsNotSuitable};

use ash::vk;

use crate::utils::Result;
use super::device::REQUIRED_EXTENSIONS;

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct DeviceScore(u32);

struct ScoredDevice {
    device: vk::PhysicalDevice,
    queue_families: QueueFamilies,
    score: DeviceScore,
}

#[derive(Default)]
pub struct QueueFamiliesBuilder {
    graphics_index: Option<usize>,
}

impl QueueFamiliesBuilder {
    fn build(&self) -> Result<QueueFamilies> {
        Ok(QueueFamilies {
            graphics_index: self.graphics_index.ok_or(PhysicalDeviceIsNotSuitable::new())? as u32
        })
    }
}

pub struct QueueFamilies {
    pub graphics_index: u32,
}


pub fn get_physical_device(instance: &ash::Instance)
                           -> Result<(vk::PhysicalDevice, QueueFamilies)> {
    unsafe { instance.enumerate_physical_devices()? }
        .into_iter()
        .filter_map(|device| {
            get_scored_device(instance, device).ok()
        })
        .max_by(|left, right| { left.score.cmp(&right.score) })
        .map(|scores_device| (scores_device.device, scores_device.queue_families))
        .ok_or(NoSuitablePhysicalDevice::new().into())
}

fn get_scored_device(instance: &ash::Instance,
                     device: vk::PhysicalDevice) -> Result<ScoredDevice> {
    let device_properties = unsafe { instance.get_physical_device_properties(device) };
    let device_features = unsafe { instance.get_physical_device_features(device) };
    let queue_families = find_queue_families(instance, device)?;

    check_device_suitability(instance, device)?;

    let score = score_device(device_properties, device_features);

    Ok(ScoredDevice { device, queue_families, score })
}

fn check_device_suitability(instance: &ash::Instance,
                            device: vk::PhysicalDevice)
                            -> Result<()> {
    check_device_available_extensions(instance, device)
}

fn get_set_of_available_device_extensions(instance: &ash::Instance,
                                          device: vk::PhysicalDevice)
                                          -> Result<HashSet<String>> {
    unsafe { instance.enumerate_device_extension_properties(device) }?
        .into_iter()
        .map(|properties| {
            Ok(properties.extension_name_as_c_str()?
                .to_str()?
                .to_string())
        })
        .collect()
}

fn check_device_available_extensions(instance: &ash::Instance,
                                     device: vk::PhysicalDevice)
                                     -> Result<()> {
    let set_of_available_extensions = get_set_of_available_device_extensions(instance,
                                                                             device)?;
    let all_required_extensions_are_available = REQUIRED_EXTENSIONS
        .iter()
        .all(|extension| {
            let Ok(extension) = unsafe { CStr::from_ptr(*extension) }.to_str() else {
                return false;
            };
            set_of_available_extensions.contains(extension)
        });

    if !all_required_extensions_are_available {
        Err(PhysicalDeviceIsNotSuitable::new())?;
    }
    Ok(())
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
                       device: vk::PhysicalDevice) -> Result<QueueFamilies> {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };

    queue_families
        .into_iter()
        .enumerate()
        .fold(QueueFamiliesBuilder::default(), |mut acc, (index, queue_family)| {
            if queue_family.queue_flags & vk::QueueFlags::GRAPHICS != vk::QueueFlags::default() {
                acc.graphics_index = Some(index);
            }
            acc
        })
        .build()
}
