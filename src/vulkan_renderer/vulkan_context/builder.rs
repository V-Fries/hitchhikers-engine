mod device;
mod instance;
#[cfg(feature = "validation_layers")]
mod validation_layers;

pub use device::SwapchainBuilder;

use super::VulkanContext;
use crate::utils::{PipeLine, Result};
use crate::vulkan_renderer::vulkan_context::queue_families::QueueFamilies;
use ash::vk;
pub use device::create_device;
pub use device::PhysicalDeviceData;
use instance::create_instance;
#[cfg(feature = "validation_layers")]
use validation_layers::{check_validation_layers, create_debug_messenger};
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[derive(Default)]
pub struct VulkanContextBuilder {
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,

    #[cfg(feature = "validation_layers")]
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

    surface: Option<vk::SurfaceKHR>,
    surface_instance: Option<ash::khr::surface::Instance>,

    physical_device_data: Option<PhysicalDeviceData>,
    device: Option<ash::Device>,
}

impl VulkanContextBuilder {
    pub unsafe fn build(mut self) -> (VulkanContext, QueueFamilies, SwapchainBuilder) {
        let physical_device_data = self.physical_device_data.take().unwrap();

        (
            VulkanContext {
                entry: self.entry.take().unwrap(),
                instance: self.instance.take().unwrap(),
                #[cfg(feature = "validation_layers")]
                debug_messenger: self.debug_messenger.take().unwrap(),
                surface_instance: self.surface_instance.take().unwrap(),
                surface: self.surface.take().unwrap(),
                device: self.device.take().unwrap(),
                is_device_destroyed: false,
                physical_device: physical_device_data.physical_device,
            },
            physical_device_data.queue_families,
            physical_device_data.swapchain_builder,
        )
    }

    pub fn create_entry(mut self) -> Result<Self, ash::LoadingError> {
        self.entry = unsafe { ash::Entry::load()? }.pipe(Some);
        Ok(self)
    }

    pub unsafe fn create_instance(mut self, display_handle: RawDisplayHandle) -> Result<Self> {
        #[cfg(feature = "validation_layers")]
        {
            check_validation_layers(self.entry())?;
        }

        self.instance = create_instance(self.entry(), display_handle)?.pipe(Some);

        #[cfg(feature = "validation_layers")]
        {
            self.debug_messenger =
                create_debug_messenger(self.entry(), self.instance())?.pipe(Some);
        }

        Ok(self)
    }

    pub unsafe fn create_surface(
        mut self,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Result<Self> {
        self.surface_instance =
            ash::khr::surface::Instance::new(self.entry(), self.instance()).pipe(Some);
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

    pub unsafe fn create_device(
        mut self,
        window_inner_size: winit::dpi::PhysicalSize<u32>,
    ) -> Result<Self> {
        self.physical_device_data = PhysicalDeviceData::new(
            self.surface_instance(),
            self.instance(),
            self.surface(),
            window_inner_size,
        )?
        .pipe(Some);

        self.device = create_device(self.instance(), self.physical_device())?.pipe(Some);

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

    unsafe fn physical_device(&self) -> &PhysicalDeviceData {
        self.physical_device_data.as_ref().unwrap()
    }

    unsafe fn surface_instance(&self) -> &ash::khr::surface::Instance {
        self.surface_instance.as_ref().unwrap()
    }
}

impl Drop for VulkanContextBuilder {
    fn drop(&mut self) {
        self.destroy_device();
        #[cfg(feature = "validation_layers")]
        {
            self.destroy_debug_messenger();
        }
        self.destroy_surface();
        self.destroy_instance();
    }
}

impl VulkanContextBuilder {
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
                self.surface_instance().destroy_surface(surface, None);
            }
        }
    }

    fn destroy_instance(&mut self) {
        if let Some(instance) = &self.instance.take() {
            unsafe { instance.destroy_instance(None) };
        }
    }
}
