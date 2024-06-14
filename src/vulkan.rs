mod errors;
#[cfg(feature = "validation_layers")]
mod validation_layers;
mod instance;
mod device;

#[cfg(feature = "validation_layers")]
use validation_layers::*;
use crate::vulkan::instance::create_instance;

use crate::utils::Result;
use winit::raw_window_handle::RawDisplayHandle;

pub struct Vulkan {
    entry: ash::Entry,
    instance: ash::Instance,
    #[cfg(feature = "validation_layers")]
    debug_messenger: ash::vk::DebugUtilsMessengerEXT,
    device: ash::Device,
    graphics_queue: ash::vk::Queue,
}

impl Vulkan {
    #[cfg(feature = "validation_layers")]
    pub fn new(display_handle: RawDisplayHandle) -> Result<Self> {
        let entry = unsafe { ash::Entry::load()? };

        check_validation_layers(&entry)?;

        let instance = create_instance(&entry, display_handle)?;

        let debug_messenger = setup_debug_messenger(&entry, &instance)
            .map_err(|err| unsafe {
                instance.destroy_instance(None);
                err
            })?;

        let (device, graphics_queue) = device::create_device(&instance)
            .map_err(|err| unsafe {
                ash::ext::debug_utils::Instance::new(&entry, &instance)
                    .destroy_debug_utils_messenger(debug_messenger, None);
                instance.destroy_instance(None);
                err
            })?;

        Ok(Self { entry, instance, debug_messenger, device, graphics_queue })
    }

    #[cfg(not(feature = "validation_layers"))]
    pub fn new(display_handle: RawDisplayHandle) -> Result<Self> {
        let entry = unsafe { ash::Entry::load()? };

        let instance = create_instance(&entry, display_handle)?;

        let (device, graphics_queue) = device::create_device(&instance)
            .map_err(|err| unsafe {
                instance.destroy_instance(None);
                err
            })?;

        Ok(Self { entry, instance, device, graphics_queue })
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
            self.instance.destroy_instance(None);
        }
    }
}
