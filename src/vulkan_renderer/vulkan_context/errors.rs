use crate::error_struct;
use std::ffi::CStr;

error_struct!(ExtensionNotFound {
    extension_name: &'static CStr,
}, "Could not find extension: {:?}", extension_name);

error_struct!(
    NoSuitablePhysicalDevice,
    "Could not find any suitable physical device"
);

error_struct!(
    PhysicalDeviceIsNotSuitable {
        device: ash::vk::PhysicalDevice,
        reason: String,
    },
    "Physical device {:?} is not suitable: {}",
    device,
    reason
);

error_struct!(ValidationLayerNotFound {
    validation_layer_name: &'static CStr,
}, "Could not find validation layer: {:?}", validation_layer_name);
