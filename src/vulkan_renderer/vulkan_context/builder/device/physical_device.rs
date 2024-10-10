use crate::vulkan_renderer::vulkan_context::errors::{
    NoSuitablePhysicalDevice, PhysicalDeviceIsNotSuitable,
};
use std::collections::HashSet;
use std::ffi::CStr;

use ash::vk;

use super::super::device::REQUIRED_EXTENSIONS;
use crate::utils::Result;
use crate::vulkan_renderer::vulkan_context::builder::device::swapchain_builder::SwapchainBuilder;
use crate::vulkan_renderer::vulkan_context::queue_families::{QueueFamilies, QueueFamiliesBuilder};

type ExtensionName = String;

pub struct PhysicalDeviceData {
    pub physical_device: vk::PhysicalDevice,
    pub queue_families: QueueFamilies,
    pub swapchain_builder: SwapchainBuilder,
}

#[derive(Ord, Eq, PartialEq, PartialOrd)]
struct DeviceScore(u32);

struct ScoredPhysicalDeviceData {
    physical_device_data: PhysicalDeviceData,
    score: DeviceScore,
}

impl PhysicalDeviceData {
    pub fn new(
        surface_instance: &ash::khr::surface::Instance,
        instance: &ash::Instance,
        surface: vk::SurfaceKHR,
        window_inner_size: winit::dpi::PhysicalSize<u32>,
    ) -> Result<PhysicalDeviceData> {
        unsafe { instance.enumerate_physical_devices()? }
            .into_iter()
            .filter_map(|device| {
                match ScoredPhysicalDeviceData::new(
                    instance,
                    &surface_instance,
                    surface,
                    window_inner_size,
                    device,
                ) {
                    Ok(scored_device) => Some(scored_device),
                    Err(err) => {
                        println!("Failed to score device {device:?}: {err}");
                        None
                    }
                }
            })
            .max_by(|left, right| left.score.cmp(&right.score))
            .map(|scored_device_data| scored_device_data.physical_device_data)
            .ok_or(NoSuitablePhysicalDevice::new().into())
    }
}

impl ScoredPhysicalDeviceData {
    fn new(
        instance: &ash::Instance,
        surface_instance: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
        window_inner_size: winit::dpi::PhysicalSize<u32>,
        device: vk::PhysicalDevice,
    ) -> Result<ScoredPhysicalDeviceData> {
        let device_properties = unsafe { instance.get_physical_device_properties(device) };
        let device_features = unsafe { instance.get_physical_device_features(device) };
        let queue_families =
            Self::find_queue_families(instance, surface_instance, surface, device)?;

        Self::check_device_suitability(instance, device)?;

        let swapchain_builder = SwapchainBuilder::new(
            device,
            queue_families,
            surface_instance,
            surface,
            window_inner_size,
        )?;

        let score = Self::score_device(device_properties, device_features);

        Ok(ScoredPhysicalDeviceData {
            physical_device_data: PhysicalDeviceData {
                physical_device: device,
                queue_families,
                swapchain_builder,
            },
            score,
        })
    }

    fn check_device_suitability(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
    ) -> Result<()> {
        Self::check_device_available_extensions(instance, device)
    }

    fn check_device_available_extensions(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
    ) -> Result<()> {
        let set_of_available_extensions =
            Self::get_set_of_available_device_extensions(instance, device)?;
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

    fn get_set_of_available_device_extensions(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
    ) -> Result<HashSet<ExtensionName>> {
        unsafe { instance.enumerate_device_extension_properties(device)? }
            .into_iter()
            .map(|properties| Ok(properties.extension_name_as_c_str()?.to_str()?.to_string()))
            .collect()
    }

    fn score_device(
        device_properties: vk::PhysicalDeviceProperties,
        _device_features: vk::PhysicalDeviceFeatures,
    ) -> DeviceScore {
        let mut score = DeviceScore(0);

        if device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score.0 += 1000;
        }

        score
    }

    fn find_queue_families(
        instance: &ash::Instance,
        surface_instance: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> Result<QueueFamilies> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };
        let has_present_queue = |index| unsafe {
            surface_instance.get_physical_device_surface_support(device, index as u32, surface)
        };

        queue_families
            .into_iter()
            .enumerate()
            .try_fold(
                QueueFamiliesBuilder::default(),
                |mut acc, (index, queue_family)| -> Result<QueueFamiliesBuilder, vk::Result> {
                    // TODO try == vk::QueueFlags::GRAPHICS
                    if queue_family.queue_flags & vk::QueueFlags::GRAPHICS
                        != vk::QueueFlags::default()
                    {
                        acc.graphics_index = Some(index);
                    }
                    if has_present_queue(index)? {
                        acc.present_index = Some(index);
                    }

                    Ok(acc)
                },
            )?
            .build(device)
    }
}
