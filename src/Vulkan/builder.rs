use ash::vk;
use crate::vulkan::{device, Vulkan};

#[derive(Default)]
pub struct VulkanBuilder {
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    #[cfg(feature = "validation_layers")]
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

    device: Option<ash::Device>,
    queues: Option<device::Queues>,

    surface: Option<vk::SurfaceKHR>,

    swap_chain: Option<vk::SwapchainKHR>,
    swap_chain_images: Option<Vec<vk::Image>>,
    swap_chain_format: Option<vk::SurfaceFormatKHR>,
    swap_chain_extent: Option<vk::Extent2D>,
}

impl VulkanBuilder {
    pub fn set_entry(&mut self, entry: ash::Entry) {
        self.entry = Some(entry);
    }

    pub fn set_instance(&mut self, instance: ash::Instance) {
        self.instance = Some(instance);
    }

    #[cfg(feature = "validation_layers")]
    pub fn set_debug_messenger(&mut self, debug_messenger: vk::DebugUtilsMessengerEXT) {
        self.debug_messenger = Some(debug_messenger);
    }

    pub fn set_device(&mut self, device: ash::Device) {
        self.device = Some(device);
    }

    pub fn set_queues(&mut self, queues: device::Queues) {
        self.queues = Some(queues);
    }

    pub fn set_surface(&mut self, surface: vk::SurfaceKHR) {
        self.surface = Some(surface);
    }

    pub fn set_swap_chain(&mut self, swap_chain: vk::SwapchainKHR) {
        self.swap_chain = Some(swap_chain);
    }

    pub fn set_swap_chain_images(&mut self, swap_chain_images: Vec<vk::Image>) {
        self.swap_chain_images = Some(swap_chain_images);
    }

    pub fn set_swap_chain_format(&mut self, swap_chain_format: vk::SurfaceFormatKHR) {
        self.swap_chain_format = Some(swap_chain_format);
    }

    pub fn set_swap_chain_extent(&mut self, swap_chain_extent: vk::Extent2D) {
        self.swap_chain_extent = Some(swap_chain_extent);
    }

    pub fn get_entry(&self) -> &ash::Entry {
        self.entry.as_ref()
            .expect("get_entry() was called before the value was initialised")
    }

    pub fn get_instance(&self) -> &ash::Instance {
        self.instance.as_ref()
            .expect("get_instance() was called before the value was initialised")
    }

    #[cfg(feature = "validation_layers")]
    pub fn get_debug_messenger(&self) -> &vk::DebugUtilsMessengerEXT {
        self.debug_messenger.as_ref()
            .expect("get_debug_messenger() was called before the value was initialised")
    }

    pub fn get_device(&self) -> &ash::Device {
        self.device.as_ref()
            .expect("get_device() was called before the value was initialised")
    }

    pub fn get_queues(&self) -> &device::Queues {
        self.queues.as_ref()
            .expect("get_queues() was called before the value was initialised")
    }

    pub fn get_surface(&self) -> vk::SurfaceKHR {
        self.surface
            .expect("get_surface() was called before the value was initialised")
    }

    pub fn get_swap_chain(&self) -> vk::SwapchainKHR {
        self.swap_chain
            .expect("get_swap_chain() was called before the value was initialised")
    }

    pub fn get_swap_chain_images(&self) -> &[vk::Image] {
        self.swap_chain_images.as_ref()
            .expect("get_swap_chain_images() was called before the value was initialised")
    }

    pub fn get_swap_chain_format(&self) -> &vk::SurfaceFormatKHR {
        self.swap_chain_format.as_ref()
            .expect("get_swap_chain_format() was called before the value was initialised")
    }

    pub fn get_swap_chain_extent(&self) -> &vk::Extent2D {
        self.swap_chain_extent.as_ref()
            .expect("get_swap_chain_extent() was called before the value was initialised")
    }

    pub fn build(mut self) -> Vulkan {
        Vulkan {
            entry: self.entry.take()
                .expect("Vulkan entry was not initialised"),
            instance: self.instance.take()
                .expect("Vulkan instance was not initialised"),
            #[cfg(feature = "validation_layers")]
            debug_messenger: self.debug_messenger.take()
                .expect("Vulkan debug_messenger was not initialised"),

            device: self.device.take()
                .expect("Vulkan device was not initialised"),
            queues: self.queues.take()
                .expect("Vulkan queues was not initialised"),

            surface: self.surface.take()
                .expect("Vulkan surface was not initialised"),

            swap_chain: self.swap_chain.take()
                .expect("Vulkan swap_chain was not initialised"),
            swap_chain_images: self.swap_chain_images.take()
                .expect("Vulkan swap_chain_images was not initialised"),
            swap_chain_format: self.swap_chain_format.take()
                .expect("Vulkan swap_chain_format was not initialised"),
            swap_chain_extent: self.swap_chain_extent.take()
                .expect("Vulkan swap_chain_extent was not initialised"),
        }
    }
}

impl Drop for VulkanBuilder {
    fn drop(&mut self) {
        let Some(entry) = &self.entry else {
            return;
        };
        let Some(instance) = &self.instance else {
            return;
        };

        unsafe {
            if let Some(swap_chain) = self.swap_chain {
                ash::khr::swapchain::Device::new(
                    instance,
                    // device must exist if swap chain does
                    self.device.as_ref()
                        .expect("Error: swap chain exist but device is None"),
                )
                    .destroy_swapchain(swap_chain, None);
            }
            if let Some(device) = &self.device {
                device.destroy_device(None);
            }
            #[cfg(feature = "validation_layers")] {
                if let Some(debug_messenger) = self.debug_messenger {
                    ash::ext::debug_utils::Instance::new(entry, instance)
                        .destroy_debug_utils_messenger(debug_messenger, None);
                }
            }
            if let Some(surface) = self.surface {
                ash::khr::surface::Instance::new(entry, instance)
                    .destroy_surface(surface, None);
            }
            instance.destroy_instance(None);
        }
    }
}
