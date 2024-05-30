mod errors;
#[cfg(feature = "validation_layers")]
mod validation_layers;
mod instance;

#[cfg(feature = "validation_layers")]
use validation_layers::*;
use crate::vulkan::instance::create_instance;

use anyhow::Result;
use winit::raw_window_handle::RawDisplayHandle;

pub struct Vulkan {
    entry: ash::Entry,
    instance: ash::Instance,
    #[cfg(feature = "validation_layers")]
    debug_messenger: ash::vk::DebugUtilsMessengerEXT,
}

impl Vulkan {
    #[cfg(feature = "validation_layers")]
    pub fn new(display_handle: RawDisplayHandle) -> Result<Self> {
        let entry = unsafe { ash::Entry::load()? };

        check_validation_layers(&entry)?;
        let instance = create_instance(&entry, display_handle)?;
        let debug_messenger = setup_debug_messenger(&entry, &instance)
            .map_err(|err| {
                unsafe { instance.destroy_instance(None) };
                err
            })?;
        Ok(Self { entry, instance, debug_messenger })
    }

    #[cfg(not(feature = "validation_layers"))]
    pub fn new(display_handle: RawDisplayHandle) -> Result<Self> {
        let entry = unsafe { ash::Entry::load()? };
        let instance = create_instance(&entry, display_handle)?;
        Ok(Self { entry, instance })
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            #[cfg(feature = "validation_layers")] {
                ash::ext::debug_utils::Instance::new(&self.entry, &self.instance)
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
