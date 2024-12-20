use ash::prelude::VkResult;
use ash::vk;
use he42_vulkan::instance::Instance;
use std::collections::HashSet;

use crate::vulkan_renderer::vulkan_context::errors::PhysicalDeviceIsNotSuitable;
use crate::vulkan_renderer::vulkan_context::queue_families::QueueFamilies;
use rs42::{
    extensions::{iterator::GetAllUniques, PipeLine},
    Result,
};

// Sorted in order of preference
const PREFERRED_FORMATS: &[vk::SurfaceFormatKHR] = &[vk::SurfaceFormatKHR {
    format: vk::Format::B8G8R8A8_SRGB,
    color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
}];

// Sorted in order of preference
const PREFERRED_PRESENTATION_MODES: &[vk::PresentModeKHR] = &[
    vk::PresentModeKHR::MAILBOX,
    vk::PresentModeKHR::FIFO,
    vk::PresentModeKHR::FIFO_RELAXED,
    vk::PresentModeKHR::IMMEDIATE,
];

const PREFERRED_IMAGE_COUNT: u32 = 3;

const NUMBER_OF_QUEUES_WORKING_ON_IMAGES: usize = 2;

pub struct SwapchainBuilder {
    capabilities: vk::SurfaceCapabilitiesKHR,
    pub format: vk::SurfaceFormatKHR,
    present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
    image_count: u32,
    queues_working_on_images: [u32; NUMBER_OF_QUEUES_WORKING_ON_IMAGES],
}

impl SwapchainBuilder {
    pub fn new(
        device: vk::PhysicalDevice,
        queue_family: QueueFamilies,
        instance: &Instance,
        surface: vk::SurfaceKHR,
        window_inner_size: winit::dpi::PhysicalSize<u32>,
    ) -> Result<Self> {
        let capabilities = unsafe {
            instance
                .surface()
                .get_physical_device_surface_capabilities(device, surface)?
        };
        Ok(Self {
            capabilities,
            format: Self::choose_surface_format(instance, device, surface)?,
            present_mode: Self::choose_present_mode(instance, device, surface)?,
            extent: Self::choose_extent(capabilities, window_inner_size),
            image_count: Self::choose_image_count(capabilities)?,
            queues_working_on_images: [queue_family.present_index, queue_family.graphics_index],
        })
    }

    pub unsafe fn build(
        &self,
        instance: &ash::Instance,
        surface: vk::SurfaceKHR,
        device: &ash::Device,
    ) -> VkResult<(vk::SwapchainKHR, ash::khr::swapchain::Device)> {
        let unique_queues = self
            .queues_working_on_images
            .into_iter()
            .get_all_uniques::<Vec<_>>();
        let create_info = self.get_create_info(surface, &unique_queues);

        let swapchain_device = ash::khr::swapchain::Device::new(instance, device);
        unsafe {
            Ok((
                swapchain_device.create_swapchain(&create_info, None)?,
                swapchain_device,
            ))
        }
    }

    fn get_create_info<'a>(
        &self,
        surface: vk::SurfaceKHR,
        unique_queues: &'a [u32],
    ) -> vk::SwapchainCreateInfoKHR<'a> {
        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(self.image_count)
            .image_format(self.format.format)
            .image_color_space(self.format.color_space)
            .image_extent(self.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(self.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(self.present_mode)
            .clipped(true); // May need to be false when accumulating ray tracing

        if self.queues_working_on_images.len() != 1
            && unique_queues.len() == self.queues_working_on_images.len()
        {
            return create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(unique_queues);
        }
        create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
    }

    fn choose_surface_format(
        instance: &Instance,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<vk::SurfaceFormatKHR> {
        let formats = Self::get_set_of_available_formats(instance, device, surface)?;

        for format in PREFERRED_FORMATS.iter() {
            if formats.contains(format) {
                return Ok(*format);
            }
        }

        let first_elem = formats
            .iter()
            .next()
            .ok_or(PhysicalDeviceIsNotSuitable::new(
                device,
                "No supported swap chain format".to_string(),
            ))?;
        println!(
            "Preferred color format is not supported, the program will try \
              running with another format"
        );
        Ok(*first_elem)
    }

    fn get_set_of_available_formats(
        instance: &Instance,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<HashSet<vk::SurfaceFormatKHR>> {
        let vec_of_available_formats = unsafe {
            instance
                .surface()
                .get_physical_device_surface_formats(device, surface)?
        };

        vec_of_available_formats
            .into_iter()
            .collect::<HashSet<vk::SurfaceFormatKHR>>()
            .pipe(Ok)
    }

    fn choose_present_mode(
        instance: &Instance,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<vk::PresentModeKHR> {
        let formats = Self::get_set_of_available_present_modes(instance, device, surface)?;

        for format in PREFERRED_PRESENTATION_MODES.iter() {
            if formats.contains(format) {
                return Ok(*format);
            }
        }

        let first_elem = formats
            .iter()
            .next()
            .ok_or(PhysicalDeviceIsNotSuitable::new(
                device,
                "No supported swap chain present mode".to_string(),
            ))?;
        println!(
            "Preferred present mode is not supported, the program will try \
              running with another present mode"
        );
        Ok(*first_elem)
    }

    fn get_set_of_available_present_modes(
        instance: &Instance,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<HashSet<vk::PresentModeKHR>> {
        let vec_of_available_formats = unsafe {
            instance
                .surface()
                .get_physical_device_surface_present_modes(device, surface)?
        };

        vec_of_available_formats
            .into_iter()
            .collect::<HashSet<vk::PresentModeKHR>>()
            .pipe(Ok)
    }

    fn choose_extent(
        capabilities: vk::SurfaceCapabilitiesKHR,
        window_inner_size: winit::dpi::PhysicalSize<u32>,
    ) -> vk::Extent2D {
        // https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain
        // according to this tutorial this should be used, but I haven't found
        // documentation proving it, so it's commented for now:
        // if capabilities.current_extent.width != u32::MAX {
        //     return capabilities.current_extent;
        // }
        vk::Extent2D::default()
            .width(window_inner_size.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ))
            .height(window_inner_size.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ))
    }

    fn choose_image_count(capabilities: vk::SurfaceCapabilitiesKHR) -> Result<u32, &'static str> {
        // according to:
        // https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain
        // Using min image count could mean we have to wait on the driver to
        // complete internal operations before we can acquire another image to
        // render to, so we try to go at least capabilities.min_image_count + 1

        if capabilities.max_image_count == capabilities.min_image_count {
            if capabilities.min_image_count == 0 {
                return Err("swap chain min_image_count can't be 0");
            }
            return Ok(capabilities.min_image_count);
        }

        if capabilities.max_image_count == 0 {
            return Ok(PREFERRED_IMAGE_COUNT.max(capabilities.min_image_count + 1));
        }

        if capabilities.max_image_count < capabilities.min_image_count {
            return Err("swap chain max_image_count is lower than min_image_count");
        }

        Ok(PREFERRED_IMAGE_COUNT.clamp(
            capabilities.min_image_count + 1,
            capabilities.max_image_count,
        ))
    }
}
