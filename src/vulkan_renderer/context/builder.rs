#[cfg(feature = "validation_layers")]
mod validation_layers;
mod instance;
mod device;

use crate::utils::{PipeLine, Result};
#[cfg(feature = "validation_layers")]
use validation_layers::{check_validation_layers, create_debug_messenger};
use device::{create_device, QueueFamilies, PhysicalDevice};
use instance::create_instance;
use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use super::Context;

#[derive(Default)]
pub struct ContextBuilder {
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,

    #[cfg(feature = "validation_layers")]
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

    surface: Option<vk::SurfaceKHR>,

    device: Option<ash::Device>,
    device_queue_families: Option<QueueFamilies>,
}

impl ContextBuilder {
    pub unsafe fn build(mut self) -> Context {
        Context {
            entry: self.entry.take().unwrap(),
            instance: self.instance.take().unwrap(),
            #[cfg(feature = "validation_layers")]
            debug_messenger: self.debug_messenger.take().unwrap(),
            surface: self.surface.take().unwrap(),
            device: self.device.take().unwrap(),
            device_queue_families: self.device_queue_families.take().unwrap(),
        }
    }


    pub fn create_entry(mut self) -> Result<Self, ash::LoadingError> {
        self.entry = unsafe { ash::Entry::load()? }
            .pipe(Some);
        Ok(self)
    }

    pub unsafe fn create_instance(mut self,
                                  display_handle: RawDisplayHandle)
                                  -> Result<Self> {
        #[cfg(feature = "validation_layers")] {
            check_validation_layers(self.entry())?;
        }

        self.instance = create_instance(self.entry(), display_handle)?
            .pipe(Some);

        #[cfg(feature = "validation_layers")] {
            self.debug_messenger = create_debug_messenger(self.entry(), self.instance())?
                .pipe(Some);
        }

        Ok(self)
    }

    pub unsafe fn create_surface(mut self,
                                 display_handle: RawDisplayHandle,
                                 window_handle: RawWindowHandle)
                                 -> Result<Self> {
        self.surface = unsafe {
            ash_window::create_surface(
                self.entry(),
                self.instance(),
                display_handle,
                window_handle,
                None,
            )?
                .pipe(Some)
        };
        Ok(self)
    }

    pub unsafe fn create_device(mut self)
                                -> Result<Self> {
        let physical_device = PhysicalDevice::new(
            self.entry(), self.instance(), self.surface(),
        )?;

        self.device = create_device(self.instance(), &physical_device)?
            .pipe(Some);

        self.device_queue_families = Some(physical_device.queue_families);

        Ok(self)
    }


    unsafe fn entry(&self) -> &ash::Entry {
        self.entry.as_ref().unwrap()
    }

    unsafe fn instance(&self) -> &ash::Instance {
        self.instance.as_ref().unwrap()
    }

    unsafe fn surface(&self) -> vk::SurfaceKHR {
        self.surface.unwrap()
    }
}

impl Drop for ContextBuilder {
    fn drop(&mut self) {
        self.destroy_device();
        #[cfg(feature = "validation_layers")] {
            self.destroy_debug_messenger();
        }
        self.destroy_surface();
        self.destroy_instance();
    }
}

impl ContextBuilder {
    fn destroy_device(&mut self) {
        if let Some(device) = &self.device.take() {
            unsafe { device.destroy_device(None) };
        }
    }

    #[cfg(feature = "validation_layers")]
    fn destroy_debug_messenger(&mut self) {
        if let Some(debug_messenger) = self.debug_messenger.take() {
            unsafe {
                ash::ext::debug_utils::Instance::new(self.entry(), self.instance())
                    .destroy_debug_utils_messenger(debug_messenger, None);
            }
        }
    }

    fn destroy_surface(&mut self) {
        if let Some(surface) = self.surface.take() {
            unsafe {
                ash::khr::surface::Instance::new(self.entry(), self.instance())
                    .destroy_surface(surface, None);
            }
        }
    }

    fn destroy_instance(&mut self) {
        if let Some(instance) = &self.instance.take() {
            unsafe { instance.destroy_instance(None) };
        }
    }
}
