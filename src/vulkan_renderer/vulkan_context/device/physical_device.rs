use crate::vulkan_renderer::vulkan_context::errors::{
    NoSuitablePhysicalDevice, PhysicalDeviceIsNotSuitable,
};
use std::collections::HashSet;
use std::ffi::CStr;

use ash::vk;
use he42_vulkan::instance::Instance;

use super::super::device::REQUIRED_EXTENSIONS;
use crate::vulkan_renderer::vulkan_context::device::swapchain_builder::SwapchainBuilder;
use crate::vulkan_renderer::vulkan_context::queue_families::{QueueFamilies, QueueFamiliesBuilder};
use rs42::Result;

type ExtensionName = String;

pub struct PhysicalDeviceData {
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub physical_device_features: vk::PhysicalDeviceFeatures,
    pub max_sample_count: vk::SampleCountFlags,
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
        instance: &Instance,
        surface: vk::SurfaceKHR,
        window_inner_size: winit::dpi::PhysicalSize<u32>,
    ) -> Result<PhysicalDeviceData> {
        unsafe { instance.enumerate_physical_devices()? }
            .into_iter()
            .filter_map(|device| {
                match ScoredPhysicalDeviceData::new(instance, surface, window_inner_size, device) {
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
        instance: &Instance,
        surface: vk::SurfaceKHR,
        window_inner_size: winit::dpi::PhysicalSize<u32>,
        device: vk::PhysicalDevice,
    ) -> Result<ScoredPhysicalDeviceData> {
        let device_properties = unsafe { instance.get_physical_device_properties(device) };
        let device_features = unsafe { instance.get_physical_device_features(device) };
        let (max_sample_count, sample_count_score) =
            Self::get_max_usable_sample_count(device_properties);
        let queue_families = Self::find_queue_families(instance, surface, device)?;

        Self::check_device_suitability(instance, device, device_features)?;

        let swapchain_builder =
            SwapchainBuilder::new(device, queue_families, instance, surface, window_inner_size)?;

        let score = Self::score_device(device_properties, device_features, sample_count_score);

        Ok(ScoredPhysicalDeviceData {
            physical_device_data: PhysicalDeviceData {
                physical_device: device,
                physical_device_properties: device_properties,
                physical_device_features: device_features,
                max_sample_count,
                queue_families,
                swapchain_builder,
            },
            score,
        })
    }

    fn check_device_suitability(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        _device_features: vk::PhysicalDeviceFeatures,
    ) -> Result<()> {
        Self::check_device_available_extensions(instance, device)?;
        Ok(())
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
        device_features: vk::PhysicalDeviceFeatures,
        sample_count_score: DeviceScore,
    ) -> DeviceScore {
        let mut score = DeviceScore(0);

        if device_properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score.0 += 1000;
        }
        if device_features.sampler_anisotropy != 0 {
            score.0 += 100;
        }

        DeviceScore(score.0 + sample_count_score.0)
    }

    fn find_queue_families(
        instance: &Instance,
        surface: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> Result<QueueFamilies> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };
        let has_present_queue = |index| unsafe {
            instance
                .surface()
                .get_physical_device_surface_support(device, index as u32, surface)
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

    fn get_max_usable_sample_count(
        device_properties: vk::PhysicalDeviceProperties,
    ) -> (vk::SampleCountFlags, DeviceScore) {
        let counts = device_properties.limits.framebuffer_color_sample_counts
            & device_properties.limits.framebuffer_depth_sample_counts;

        let is_supported = |test_counts| counts & test_counts != vk::SampleCountFlags::empty();

        if is_supported(vk::SampleCountFlags::TYPE_64) {
            return (vk::SampleCountFlags::TYPE_64, DeviceScore(64));
        }
        if is_supported(vk::SampleCountFlags::TYPE_32) {
            return (vk::SampleCountFlags::TYPE_32, DeviceScore(32));
        }
        if is_supported(vk::SampleCountFlags::TYPE_16) {
            return (vk::SampleCountFlags::TYPE_16, DeviceScore(16));
        }
        if is_supported(vk::SampleCountFlags::TYPE_8) {
            return (vk::SampleCountFlags::TYPE_8, DeviceScore(8));
        }
        if is_supported(vk::SampleCountFlags::TYPE_8) {
            return (vk::SampleCountFlags::TYPE_8, DeviceScore(8));
        }
        if is_supported(vk::SampleCountFlags::TYPE_4) {
            return (vk::SampleCountFlags::TYPE_4, DeviceScore(4));
        }
        if is_supported(vk::SampleCountFlags::TYPE_2) {
            return (vk::SampleCountFlags::TYPE_2, DeviceScore(2));
        }
        (vk::SampleCountFlags::TYPE_1, DeviceScore(1))
    }
}
