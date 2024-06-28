use std::collections::HashSet;
use std::ffi::CStr;
use crate::vulkan::errors::{NoSuitablePhysicalDevice, PhysicalDeviceIsNotSuitable};

use ash::vk;

use crate::utils::{GetAllUniques, PipeLine, Result};
use crate::vulkan::swap_chain::SwapChainBuilder;
use super::device::REQUIRED_EXTENSIONS;

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct DeviceScore(u32);

pub struct DeviceData {
    pub physical_device: vk::PhysicalDevice,
    pub queue_families: QueueFamilies,
    pub swap_chain_builder: SwapChainBuilder,
    score: DeviceScore,
}

#[derive(Default)]
pub struct QueueFamiliesBuilder {
    graphics_index: Option<usize>,
    present_index: Option<usize>,
}

impl QueueFamiliesBuilder {
    fn build(&self, device: vk::PhysicalDevice) -> Result<QueueFamilies> {
        let option_to_u32 = |option: Option<usize>, queue_name: &str|
                             -> Result<u32, PhysicalDeviceIsNotSuitable> {
            option
                .ok_or(PhysicalDeviceIsNotSuitable::new(
                    device,
                    format!("{queue_name} queue is not supported"),
                ))?
                .pipe(|p| p as u32)
                .pipe(Ok)
        };

        Ok(QueueFamilies {
            graphics_index: option_to_u32(self.graphics_index, "graphics")?,
            present_index: option_to_u32(self.present_index, "present")?,
        })
    }
}

#[derive(Clone, Copy)]
pub struct QueueFamilies {
    pub graphics_index: u32,
    pub present_index: u32,
}

impl QueueFamilies {
    pub fn as_vec_of_unique_indexes(&self) -> Vec<u32> {
        [self.graphics_index, self.present_index].into_iter().get_all_uniques()
    }
}


pub fn pick_physical_device(entry: &ash::Entry,
                            instance: &ash::Instance,
                            surface: vk::SurfaceKHR,
                            window_inner_size: winit::dpi::PhysicalSize<u32>)
                            -> Result<DeviceData> {
    let surface_instance = ash::khr::surface::Instance::new(entry, instance);

    unsafe { instance.enumerate_physical_devices()? }
        .into_iter()
        .filter_map(|device| {
            match get_device_data(
                instance, &surface_instance, surface, window_inner_size, device,
            ) {
                Ok(scored_device) => Some(scored_device),
                Err(err) => {
                    println!("Failed to score device {device:?}: {err}");
                    None
                }
            }
        })
        .max_by(|left, right| { left.score.cmp(&right.score) })
        .ok_or(NoSuitablePhysicalDevice::new().into())
}

fn get_device_data(instance: &ash::Instance,
                   surface_instance: &ash::khr::surface::Instance,
                   surface: vk::SurfaceKHR,
                   window_inner_size: winit::dpi::PhysicalSize<u32>,
                   device: vk::PhysicalDevice) -> Result<DeviceData> {
    let device_properties = unsafe { instance.get_physical_device_properties(device) };
    let device_features = unsafe { instance.get_physical_device_features(device) };
    let queue_families = find_queue_families(instance, surface_instance, surface, device)?;

    check_device_suitability(instance, device)?;

    let swap_chain_builder = SwapChainBuilder::new(
        device, queue_families, surface_instance, surface, window_inner_size,
    )?;
    let score = score_device(device_properties, device_features);

    Ok(DeviceData {
        physical_device: device,
        queue_families,
        swap_chain_builder,
        score,
    })
}

fn check_device_suitability(instance: &ash::Instance,
                            device: vk::PhysicalDevice)
                            -> Result<()> {
    check_device_available_extensions(instance, device)
}

type ExtensionName = String;

fn get_set_of_available_device_extensions(instance: &ash::Instance,
                                          device: vk::PhysicalDevice)
                                          -> Result<HashSet<ExtensionName>> {
    unsafe { instance.enumerate_device_extension_properties(device)? }
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
    for extension in REQUIRED_EXTENSIONS.iter() {
        let extension = unsafe { CStr::from_ptr(*extension).to_str()? };
        if !set_of_available_extensions.contains(extension) {
            Err(PhysicalDeviceIsNotSuitable::new(
                device,
                format!("extension {extension} is not supported"),
            ))?;
        }
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
                       surface_instance: &ash::khr::surface::Instance,
                       surface: vk::SurfaceKHR,
                       device: vk::PhysicalDevice) -> Result<QueueFamilies> {
    let queue_families = unsafe {
        instance.get_physical_device_queue_family_properties(device)
    };
    let has_present_queue = |index| unsafe {
        surface_instance.get_physical_device_surface_support(
            device, index as u32, surface,
        )
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
        .build(device)
}
