use std::collections::{BTreeSet, HashSet};
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
    present_index: Option<usize>,
}

impl QueueFamiliesBuilder {
    fn build(&self) -> Result<QueueFamilies> {
        let option_to_u32 = |option: Option<usize>|
                             -> Result<u32, PhysicalDeviceIsNotSuitable> {
            Ok(option.ok_or(PhysicalDeviceIsNotSuitable::new())? as u32)
        };

        Ok(QueueFamilies {
            graphics_index: option_to_u32(self.graphics_index)?,
            present_index: option_to_u32(self.present_index)?,
        })
    }
}

pub struct QueueFamilies {
    pub graphics_index: u32,
    pub present_index: u32,
}

impl QueueFamilies {
    pub fn as_vec_of_unique_indexes(&self) -> Vec<u32> {
        BTreeSet::from([self.graphics_index, self.present_index])
            .into_iter()
            .collect()
    }
}


pub fn get_physical_device(entry: &ash::Entry,
                           instance: &ash::Instance,
                           surface: vk::SurfaceKHR)
                           -> Result<(vk::PhysicalDevice, QueueFamilies)> {
    unsafe { instance.enumerate_physical_devices()? }
        .into_iter()
        .filter_map(|device| {
            get_scored_device(entry, instance, surface, device).ok()
        })
        .max_by(|left, right| { left.score.cmp(&right.score) })
        .map(|scores_device| (scores_device.device, scores_device.queue_families))
        .ok_or(NoSuitablePhysicalDevice::new().into())
}

fn get_scored_device(entry: &ash::Entry,
                     instance: &ash::Instance,
                     surface: vk::SurfaceKHR,
                     device: vk::PhysicalDevice) -> Result<ScoredDevice> {
    let device_properties = unsafe { instance.get_physical_device_properties(device) };
    let device_features = unsafe { instance.get_physical_device_features(device) };
    let queue_families = find_queue_families(entry, instance, surface, device)?;

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

fn find_queue_families(entry: &ash::Entry,
                       instance: &ash::Instance,
                       surface: vk::SurfaceKHR,
                       device: vk::PhysicalDevice) -> Result<QueueFamilies> {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(device) };
    let surface_instance = ash::khr::surface::Instance::new(entry, instance);
    let has_present_queue = |index| unsafe {
        surface_instance.get_physical_device_surface_support(device, index as u32, surface)
    };

    queue_families
        .into_iter()
        .enumerate()
        .try_fold(QueueFamiliesBuilder::default(), |mut acc, (index, queue_family)|
                                                    -> Result<QueueFamiliesBuilder, vk::Result> {
            if queue_family.queue_flags & vk::QueueFlags::GRAPHICS != vk::QueueFlags::default() {
                acc.graphics_index = Some(index);
            }
            if has_present_queue(index)? {
                acc.present_index = Some(index);
            }

            Ok(acc)
        })?
        .build()
}
