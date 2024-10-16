use crate::utils::Result;
use ash::vk;

use super::super::errors::ValidationLayerNotFound;
use std::collections::HashSet;
use std::ffi::{c_char, CStr};

type LayerName = String;

pub const VALIDATION_LAYERS: &[*const c_char] = &[c"VK_LAYER_KHRONOS_validation".as_ptr()];

pub fn check_validation_layers(entry: &ash::Entry) -> Result<()> {
    let available_layers = get_set_of_available_validation_layers(entry)?;

    for layer in VALIDATION_LAYERS {
        let layer = unsafe { CStr::from_ptr(*layer) };
        if !available_layers.contains(layer.to_str()?) {
            Err(ValidationLayerNotFound::new(layer))?;
        }
    }
    Ok(())
}

fn get_set_of_available_validation_layers(entry: &ash::Entry) -> Result<HashSet<LayerName>> {
    unsafe { entry.enumerate_instance_layer_properties()? }
        .into_iter()
        .map::<Result<String>, _>(|elem| Ok(elem.layer_name_as_c_str()?.to_str()?.to_string()))
        .collect()
}

pub fn create_debug_messenger(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> Result<vk::DebugUtilsMessengerEXT> {
    let create_info = get_debug_utils_messenger_create_info();
    let debug_utils = ash::ext::debug_utils::Instance::new(entry, instance);
    unsafe { Ok(debug_utils.create_debug_utils_messenger(&create_info, None)?) }
}

fn get_debug_utils_messenger_create_info<'a>() -> vk::DebugUtilsMessengerCreateInfoEXT<'a> {
    vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(debug_utils_messenger_callback))
}

unsafe extern "system" fn debug_utils_messenger_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Validation layer][{message_severity:?}][{message_type:?}]: {message:?}\n");
    vk::FALSE
}
