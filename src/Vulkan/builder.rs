use ash::prelude::VkResult;
use ash::vk;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use crate::vulkan::{device, Vulkan};
use crate::vulkan::instance::create_instance;
#[cfg(feature = "validation_layers")]
use crate::vulkan::validation_layers::{check_validation_layers, setup_debug_messenger};
use crate::utils::{PipeLine, Result};
use crate::vulkan::device::{create_device, create_device_queue, DeviceData, pick_physical_device};
use crate::vulkan::image_views::create_image_views;
use crate::vulkan::shader::{FRAG_SHADER_PATH, ShaderModule, VERT_SHADER_PATH};

#[derive(Default)]
pub struct VulkanBuilder {
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    #[cfg(feature = "validation_layers")]
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

    device_data: Option<DeviceData>,
    device: Option<ash::Device>,
    queues: Option<device::Queues>,

    surface: Option<vk::SurfaceKHR>,

    swap_chain: Option<vk::SwapchainKHR>,
    swap_chain_images: Option<Vec<vk::Image>>,
    swap_chain_format: Option<vk::Format>,
    swap_chain_extent: Option<vk::Extent2D>,
    image_views: Option<Vec<vk::ImageView>>,
}

impl VulkanBuilder {
    pub fn new(display_handle: RawDisplayHandle,
               window_handle: RawWindowHandle,
               window_inner_size: winit::dpi::PhysicalSize<u32>)
               -> Result<Self> {
        let mut builder = Self::default()
            .create_entry()?;

        #[cfg(feature = "validation_layers")] {
            check_validation_layers(builder.entry())?;
        }

        builder = builder.create_instance(display_handle)?;

        #[cfg(feature = "validation_layers")] {
            builder = builder.create_debug_messenger()?;
        }

        builder
            .create_surface(display_handle, window_handle)?
            .create_device(window_inner_size)?
            .create_queues()
            .create_swap_chain()?
            .create_image_views()?
            .create_graphics_pipeline()?
            .pipe(Ok)
    }

    fn create_entry(mut self) -> Result<Self, ash::LoadingError> {
        self.entry = unsafe { ash::Entry::load() }?
            .pipe(Some);
        Ok(self)
    }

    fn create_instance(mut self,
                       display_handle: RawDisplayHandle)
                       -> Result<Self> {
        self.instance = create_instance(self.entry(), display_handle)?
            .pipe(Some);
        Ok(self)
    }

    #[cfg(feature = "validation_layers")]
    fn create_debug_messenger(mut self) -> Result<Self> {
        self.debug_messenger = setup_debug_messenger(self.entry(),
                                                     self.instance())?
            .pipe(Some);
        Ok(self)
    }

    fn create_surface(mut self,
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

    fn create_device(mut self,
                     window_inner_size: winit::dpi::PhysicalSize<u32>)
                     -> Result<Self> {
        self.device_data = pick_physical_device(
            self.entry(), self.instance(), self.surface(),
            window_inner_size,
        )?
            .pipe(Some);

        self.device = create_device(self.instance(), self.device_data())?
            .pipe(Some);

        Ok(self)
    }

    fn create_queues(mut self) -> Self {
        self.queues = create_device_queue(self.device(), self.device_data())
            .pipe(Some);
        self
    }

    fn create_swap_chain(mut self) -> Result<Self> {
        let swap_chain_builder = self.take_device_data().swap_chain_builder;

        self.swap_chain = swap_chain_builder.build(
            self.instance(), self.surface(), self.device(),
        )?
            .pipe(Some);

        self.swap_chain_images = unsafe {
            ash::khr::swapchain::Device::new(
                self.instance(), self.device(),
            )
                .get_swapchain_images(self.swap_chain())?
                .pipe(Some)
        };

        self.swap_chain_format = Some(swap_chain_builder.format.format);
        self.swap_chain_extent = Some(swap_chain_builder.extent);
        Ok(self)
    }

    fn create_image_views(mut self) -> VkResult<Self> {
        self.image_views = create_image_views(self.device(),
                                              self.swap_chain_images(),
                                              self.swap_chain_format())?
            .pipe(Some);
        Ok(self)
    }

    fn create_graphics_pipeline(self) -> Result<Self> {
        let vert_shader_module = ShaderModule::new(self.device(), VERT_SHADER_PATH)?;
        let frag_shader_module = ShaderModule::new(self.device(), FRAG_SHADER_PATH)?;

        drop(vert_shader_module);
        drop(frag_shader_module);
        Ok(self)
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
            image_views: self.image_views.take()
                .expect("Vulkan image_views was not initialised"),
        }
    }


    fn entry(&self) -> &ash::Entry {
        self.entry.as_ref()
            .expect("get_entry() was called before the value was initialised")
    }

    fn instance(&self) -> &ash::Instance {
        self.instance.as_ref()
            .expect("get_instance() was called before the value was initialised")
    }

    #[cfg(feature = "validation_layers")]
    fn debug_messenger(&self) -> &vk::DebugUtilsMessengerEXT {
        self.debug_messenger.as_ref()
            .expect("get_debug_messenger() was called before the value was initialised")
    }

    fn device(&self) -> &ash::Device {
        self.device.as_ref()
            .expect("get_device() was called before the value was initialised")
    }

    fn queues(&self) -> &device::Queues {
        self.queues.as_ref()
            .expect("get_queues() was called before the value was initialised")
    }

    fn surface(&self) -> vk::SurfaceKHR {
        self.surface
            .expect("get_surface() was called before the value was initialised")
    }

    fn device_data(&self) -> &DeviceData {
        self.device_data.as_ref()
            .expect("get_swap_chain_builder() was called before the value was initialised")
    }

    fn take_device_data(&mut self) -> DeviceData {
        self.device_data.take()
            .expect("take_device_data() was called before the value was initialised")
    }

    fn swap_chain(&self) -> vk::SwapchainKHR {
        self.swap_chain
            .expect("get_swap_chain() was called before the value was initialised")
    }

    fn swap_chain_images(&self) -> &[vk::Image] {
        self.swap_chain_images.as_ref()
            .expect("get_swap_chain_images() was called before the value was initialised")
    }

    fn swap_chain_format(&self) -> vk::Format {
        self.swap_chain_format
            .expect("get_swap_chain_format() was called before the value was initialised")
    }

    fn swap_chain_extent(&self) -> &vk::Extent2D {
        self.swap_chain_extent.as_ref()
            .expect("get_swap_chain_extent() was called before the value was initialised")
    }
}

impl Drop for VulkanBuilder {
    fn drop(&mut self) {
        self.destroy_image_views();
        self.destroy_swap_chain();
        self.destroy_device();
        #[cfg(feature = "validation_layers")] {
            self.destroy_debug_messenger();
        }
        self.destroy_surface();
        self.destroy_instance();
    }
}

impl VulkanBuilder {
    fn destroy_image_views(&mut self) {
        if let Some(image_views) = &self.image_views {
            let device = self.device();
            for image_view in image_views {
                unsafe { device.destroy_image_view(*image_view, None) };
            }
        }
    }

    fn destroy_swap_chain(&mut self) {
        if let Some(swap_chain) = self.swap_chain {
            unsafe {
                ash::khr::swapchain::Device::new(self.instance(), self.device())
                    .destroy_swapchain(swap_chain, None);
            }
        }
    }

    fn destroy_device(&mut self) {
        if let Some(device) = &self.device {
            unsafe { device.destroy_device(None) };
        }
    }

    #[cfg(feature = "validation_layers")]
    fn destroy_debug_messenger(&mut self) {
        if let Some(debug_messenger) = self.debug_messenger {
            unsafe {
                ash::ext::debug_utils::Instance::new(self.entry(), self.instance())
                    .destroy_debug_utils_messenger(debug_messenger, None);
            }
        }
    }

    fn destroy_surface(&mut self) {
        if let Some(surface) = self.surface {
            unsafe {
                ash::khr::surface::Instance::new(self.entry(), self.instance())
                    .destroy_surface(surface, None);
            }
        }
    }

    fn destroy_instance(&mut self) {
        if let Some(instance) = &self.instance {
            unsafe { instance.destroy_instance(None) };
        }
    }
}
