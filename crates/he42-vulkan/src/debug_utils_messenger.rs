use std::sync::Arc;

use ash::vk;

use crate::instance::Instance;
use crate::VkResult;

pub type DebugUtilsMessengerCreateInfo<'a> = vk::DebugUtilsMessengerCreateInfoEXT<'a>;

pub struct DebugUtilsMessenger {
    instance: Arc<Instance>,
    raw_messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugUtilsMessenger {
    /// # Safety
    ///
    /// Create info must contain valid data (see vulkan specification)
    ///
    /// The application must ensure that DebugUtilsMessenger::new() is not executed in parallel
    /// with any Vulkan command that is also called with instance or child of instance as the
    /// dispatchable argument.
    pub unsafe fn new(
        instance: Arc<Instance>,
        create_info: DebugUtilsMessengerCreateInfo,
    ) -> VkResult<Self> {
        Ok(Self {
            raw_messenger: instance
                .debug_utils()
                .create_debug_utils_messenger(&create_info, None)?,
            instance,
        })
    }
}

impl Drop for DebugUtilsMessenger {
    fn drop(&mut self) {
        unsafe {
            self.instance
                .debug_utils()
                .destroy_debug_utils_messenger(self.raw_messenger, None);
        }
    }
}
