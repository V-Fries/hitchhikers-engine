mod errors;
#[cfg(feature = "validation_layers")]
mod validation_layers;
mod instance;
mod device;

use ash::vk;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[cfg(feature = "validation_layers")]
use validation_layers::*;
use crate::vulkan::instance::create_instance;
use crate::utils::Result;

pub struct Vulkan {
    entry: ash::Entry,
    instance: ash::Instance,
    #[cfg(feature = "validation_layers")]
    debug_messenger: vk::DebugUtilsMessengerEXT,

    device: ash::Device,
    queues: device::Queues,

    surface: vk::SurfaceKHR,
}

impl Vulkan {
    #[cfg(feature = "validation_layers")]
    pub fn new(display_handle: RawDisplayHandle,
               window_handle: RawWindowHandle) -> Result<Self> {
        let entry = unsafe { ash::Entry::load()? };

        check_validation_layers(&entry)?;

        let instance = create_instance(&entry, display_handle)?;

        let debug_messenger = setup_debug_messenger(&entry, &instance)
            .map_err(|err| unsafe {
                instance.destroy_instance(None);
                err
            })?;

        let surface = unsafe {
            ash_window::create_surface(
                &entry, &instance, display_handle, window_handle, None,
            )
                .map_err(|err| {
                    ash::ext::debug_utils::Instance::new(&entry, &instance)
                        .destroy_debug_utils_messenger(debug_messenger, None);
                    instance.destroy_instance(None);
                    err
                })?
        };

        let (device, queues) = device::create_device(&entry, &instance, surface)
            .map_err(|err| unsafe {
                ash::ext::debug_utils::Instance::new(&entry, &instance)
                    .destroy_debug_utils_messenger(debug_messenger, None);
                ash::khr::surface::Instance::new(&entry, &instance)
                    .destroy_surface(surface, None);
                instance.destroy_instance(None);
                err
            })?;

        Ok(Self { entry, instance, debug_messenger, device, queues, surface })
    }

    #[cfg(not(feature = "validation_layers"))]
    pub fn new(display_handle: RawDisplayHandle,
               window_handle: RawWindowHandle) -> Result<Self> {
        let entry = unsafe { ash::Entry::load()? };

        let instance = create_instance(&entry, display_handle)?;

        let surface = unsafe {
            ash_window::create_surface(
                &entry, &instance, display_handle, window_handle, None,
            )
                .map_err(|err| {
                    instance.destroy_instance(None);
                    err
                })?
        };

        let (device, queues) = device::create_device(&entry, &instance, surface)
            .map_err(|err| unsafe {
                ash::khr::surface::Instance::new(&entry, &instance)
                    .destroy_surface(surface, None);
                instance.destroy_instance(None);
                err
            })?;

        Ok(Self { entry, instance, device, queues, surface })
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            #[cfg(feature = "validation_layers")] {
                ash::ext::debug_utils::Instance::new(&self.entry, &self.instance)
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            ash::khr::surface::Instance::new(&self.entry, &self.instance)
                .destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
