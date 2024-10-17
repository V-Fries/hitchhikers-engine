mod device;
mod errors;
mod instance;
mod queue_families;
mod validation_layers;

use crate::utils::{Defer, Result, ScopeGuard};
use ash::{prelude::VkResult, vk};
pub use device::{create_device, PhysicalDeviceData, SwapchainBuilder};
use instance::create_instance;
pub use queue_families::QueueFamilies;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct VulkanContext {
    #[allow(dead_code)]
    entry: ash::Entry,
    instance: ash::Instance,

    #[cfg(feature = "validation_layers")]
    debug_messenger: vk::DebugUtilsMessengerEXT,

    surface: vk::SurfaceKHR,
    surface_instance: ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    is_device_destroyed: bool,
}

impl VulkanContext {
    pub fn new(window: &winit::window::Window) -> Result<(Self, QueueFamilies, SwapchainBuilder)> {
        let display_handle = window.display_handle()?.into();
        let window_handle = window.window_handle()?.into();

        let entry = unsafe { ash::Entry::load()? };

        let (instance, debug_messenger) = create_instance(&entry, display_handle)?;
        let instance = instance.defer(|instance| unsafe { instance.destroy_instance(None) });
        let debug_messenger = debug_messenger.defer(|debug_messenger| unsafe {
            if let Some(debug_messenger) = debug_messenger {
                ash::ext::debug_utils::Instance::new(&entry, &instance)
                    .destroy_debug_utils_messenger(debug_messenger, None);
            }
        });

        let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);

        let surface = unsafe {
            ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)?
                .defer(|surface| surface_instance.destroy_surface(surface, None))
        };

        let physical_device_data =
            PhysicalDeviceData::new(&surface_instance, &instance, *surface, window.inner_size())?;
        let device = unsafe { create_device(&instance, &physical_device_data)? }
            .defer(|device| unsafe { device.destroy_device(None) });

        #[cfg(not(feature = "validation_layers"))]
        {
            let _ = ScopeGuard::into_inner(debug_messenger);
        }
        Ok((
            VulkanContext {
                device: ScopeGuard::into_inner(device),
                physical_device: physical_device_data.physical_device,
                surface: ScopeGuard::into_inner(surface),
                surface_instance,
                #[cfg(feature = "validation_layers")]
                debug_messenger: ScopeGuard::into_inner(debug_messenger).unwrap(),
                instance: ScopeGuard::into_inner(instance),
                entry,
                is_device_destroyed: false,
            },
            physical_device_data.queue_families,
            physical_device_data.swapchain_builder,
        ))
    }

    pub fn device_wait_idle(&self) -> VkResult<()> {
        if self.is_device_destroyed {
            return Ok(());
        }

        unsafe { self.device.device_wait_idle() }
    }

    pub fn device(&self) -> &ash::Device {
        debug_assert!(
            !self.is_device_destroyed,
            "VulkanContext::device() was called after device destruction"
        );
        &self.device
    }

    pub fn set_device(&mut self, device: ash::Device, physical_device: vk::PhysicalDevice) {
        // TODO maybe make a Device struct that holds both those values?

        debug_assert!(
            self.is_device_destroyed,
            "VulkanContext::set_device() was called without device destruction"
        );

        self.physical_device = physical_device;
        self.device = device;
        self.is_device_destroyed = false;
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        debug_assert!(
            !self.is_device_destroyed,
            "VulkanContext::physical_device() was called after device destruction"
        );

        self.physical_device
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn surface_instance(&self) -> &ash::khr::surface::Instance {
        &self.surface_instance
    }

    pub unsafe fn destroy_device(&mut self) {
        debug_assert!(
            !self.is_device_destroyed,
            "VulkanContext::destroy_device() was called after device destruction"
        );
        self.device.destroy_device(None);
        self.is_device_destroyed = true;
    }

    pub unsafe fn destroy(&mut self) {
        // TODO add is_destroyed member and do debug_assertions with it (Same for other structs)

        if !self.is_device_destroyed {
            self.destroy_device();
        }
        #[cfg(feature = "validation_layers")]
        {
            ash::ext::debug_utils::Instance::new(&self.entry, &self.instance)
                .destroy_debug_utils_messenger(self.debug_messenger, None);
        }
        self.surface_instance.destroy_surface(self.surface, None);
        self.instance.destroy_instance(None);
    }
}
