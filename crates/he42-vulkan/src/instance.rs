use super::VkResult;
use super::VulkanLibrary;
use rs42::extensions::PipeLine;
use std::ops::Deref;
use std::sync::Arc;

pub use ash::vk::InstanceCreateInfo;

type DebugUtilsInstance = ash::ext::debug_utils::Instance;
type SurfaceInstance = ash::khr::surface::Instance;

pub struct Instance {
    vulkan_library: Arc<VulkanLibrary>,

    raw_instance: ash::Instance,

    debug_utils: DebugUtilsInstance,
    surface: SurfaceInstance,
}

impl Instance {
    /// # Safety
    ///
    /// All required extensions for each extension in the CreateInfo::EnabledExtensionNames list
    /// must also be present in that list
    pub unsafe fn new(
        vulkan_library: Arc<VulkanLibrary>,
        instance_create_info: ash::vk::InstanceCreateInfo,
    ) -> VkResult<Arc<Self>> {
        vulkan_library
            .create_instance(&instance_create_info, None)?
            .pipe(|raw_instance| Instance {
                debug_utils: DebugUtilsInstance::new(&vulkan_library, &raw_instance),
                surface: SurfaceInstance::new(&vulkan_library, &raw_instance),
                vulkan_library,
                raw_instance,
            })
            .pipe(Arc::new)
            .pipe(Ok)
    }

    pub fn debug_utils(&self) -> &DebugUtilsInstance {
        &self.debug_utils
    }

    pub fn surface(&self) -> &SurfaceInstance {
        &self.surface
    }

    pub fn raw_instance(&self) -> &ash::Instance {
        &self.raw_instance
    }

    pub fn vulkan_library(&self) -> &Arc<VulkanLibrary> {
        &self.vulkan_library
    }
}

impl Deref for Instance {
    type Target = ash::Instance;

    fn deref(&self) -> &Self::Target {
        &self.raw_instance
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.raw_instance.destroy_instance(None) }
    }
}
