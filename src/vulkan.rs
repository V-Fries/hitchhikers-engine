mod errors;

use ash::vk;
use std::collections::HashSet;
use std::ffi::{c_char, CStr};
use anyhow::Result;
use ash::vk::ApplicationInfo;
use errors::ExtensionNotFound;

#[cfg(feature = "validation_layers")]
use validation_layers::*;

#[cfg(feature = "validation_layers")]
use errors::validation_layers::ValidationLayerNotFound;

const ENGINE_NAME: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"HitchHiker's Engine\0") };
const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);

const REQUIRED_EXTENSIONS: &[&CStr] = &[
    vk::KHR_PORTABILITY_ENUMERATION_NAME,
    #[cfg(feature = "validation_layers")]
        vk::EXT_DEBUG_UTILS_NAME,
];

#[cfg(feature = "validation_layers")]
const VALIDATION_LAYERS: &[*const c_char] = &[
    unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0").as_ptr() },
];


pub struct Vulkan {
    entry: ash::Entry,
    instance: ash::Instance,
    #[cfg(feature = "validation_layers")]
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Vulkan {
    #[cfg(feature = "validation_layers")]
    pub fn new() -> Result<Self> {
        let entry = unsafe { ash::Entry::load()? };

        check_validation_layers(&entry)?;
        let instance = create_instance(&entry)?;
        let debug_messenger = setup_debug_messenger(&entry, &instance)
            .map_err(|err| {
                unsafe { instance.destroy_instance(None) };
                err
            })?;
        Ok(Self { entry, instance, debug_messenger })
    }

    #[cfg(not(feature = "validation_layers"))]
    pub fn new() -> Result<Self> {
        let entry = unsafe { ash::Entry::load()? };
        let instance = create_instance(&entry)?;
        Ok(Self { entry, instance })
    }
}

impl Drop for Vulkan {
    #[cfg(feature = "validation_layers")]
    fn drop(&mut self) {
        unsafe {
            ash::ext::debug_utils::Instance::new(&self.entry, &self.instance)
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }

    #[cfg(not(feature = "validation_layers"))]
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}

fn create_instance(entry: &ash::Entry) -> Result<ash::Instance> {
    let available_extensions = get_set_of_available_extensions(entry)?;
    let required_extensions = get_required_extensions(available_extensions)?;

    let app_info = get_app_info();
    let create_info = get_create_info(&required_extensions, &app_info);

    Ok(unsafe { entry.create_instance(&create_info, None)? })
}

fn get_set_of_available_extensions(entry: &ash::Entry) -> Result<HashSet<String>> {
    unsafe { entry.enumerate_instance_extension_properties(None)? }
        .into_iter()
        .map::<Result<String>, _>(|elem| {
            Ok(elem.extension_name_as_c_str()?
                .to_str()?
                .to_string())
        })
        .collect()
}

fn get_required_extensions(available_extensions: HashSet<String>)
                           -> Result<Vec<*const c_char>> {
    REQUIRED_EXTENSIONS
        .iter()
        .map::<Result<*const c_char>, _>(|extension| {
            if !available_extensions.contains(extension.to_str()?) {
                Err(ExtensionNotFound::new(extension))?;
            }
            Ok(extension.as_ptr())
        })
        .collect()
}

fn get_app_info<'a>() -> ApplicationInfo<'a> {
    ApplicationInfo::default()
        .api_version(vk::API_VERSION_1_3)
        .application_name(ENGINE_NAME)
        .application_version(ENGINE_VERSION)
        .engine_name(ENGINE_NAME)
        .engine_version(ENGINE_VERSION)
}

fn get_create_info<'a>(required_extensions: &'a [*const c_char],
                       app_info: &'a ApplicationInfo)
                       -> vk::InstanceCreateInfo<'a> {
    #[allow(unused_mut)]
        let mut create_info = vk::InstanceCreateInfo::default()
        .application_info(app_info)
        .flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
        .enabled_extension_names(required_extensions);
    #[cfg(feature = "validation_layers")] {
        create_info = create_info.enabled_layer_names(VALIDATION_LAYERS);
    }
    create_info
}

#[cfg(feature = "validation_layers")]
mod validation_layers {
    use super::*;

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

    fn get_set_of_available_validation_layers(entry: &ash::Entry)
                                              -> Result<HashSet<String>> {
        unsafe { entry.enumerate_instance_layer_properties()? }
            .into_iter()
            .map::<Result<String>, _>(|elem| {
                Ok(elem.layer_name_as_c_str()?
                    .to_str()?
                    .to_string())
            })
            .collect()
    }

    pub fn setup_debug_messenger(entry: &ash::Entry, instance: &ash::Instance) -> Result<vk::DebugUtilsMessengerEXT> {
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
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE)
            .pfn_user_callback(Some(debug_utils_messenger_callback))
    }

    unsafe extern "system" fn debug_utils_messenger_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut std::ffi::c_void,
    ) -> vk::Bool32 {
        let message = CStr::from_ptr((*p_callback_data).p_message);
        println!("[Debug][{message_severity:?}][{message_type:?}]: {message:?}\n");
        vk::FALSE
    }
}
