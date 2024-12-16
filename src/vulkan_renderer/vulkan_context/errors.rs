use rs42::error_struct_custom_display;
use std::ffi::CStr;

error_struct_custom_display!(ExtensionNotFound {
    extension_name: &'static CStr,
}, "Could not find extension: {:?}", extension_name);

error_struct_custom_display!(
    NoSuitablePhysicalDevice,
    "Could not find any suitable physical device"
);

error_struct_custom_display!(
    PhysicalDeviceIsNotSuitable {
        device: ash::vk::PhysicalDevice,
        reason: String,
    },
    "Physical device {:?} is not suitable: {}",
    device,
    reason
);

error_struct_custom_display!(ValidationLayerNotFound {
    validation_layer_name: &'static CStr,
}, "Could not find validation layer: {:?}", validation_layer_name);
