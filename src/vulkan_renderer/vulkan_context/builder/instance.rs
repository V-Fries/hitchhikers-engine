use super::super::errors::ExtensionNotFound;
#[cfg(feature = "validation_layers")]
use super::validation_layers::VALIDATION_LAYERS;
use crate::engine::{ENGINE_NAME_CSTR, ENGINE_VERSION};

use crate::utils::{PipeLine, Result};
use ash::vk;

use std::collections::HashSet;
use std::ffi::{c_char, CStr};
use winit::raw_window_handle::RawDisplayHandle;

type ExtensionName = String;

const REQUIRED_EXTENSIONS: &[&CStr] = &[
    vk::KHR_PORTABILITY_ENUMERATION_NAME,
    #[cfg(feature = "validation_layers")]
    vk::EXT_DEBUG_UTILS_NAME,
];

pub fn create_instance(
    entry: &ash::Entry,
    display_handle: RawDisplayHandle,
) -> Result<ash::Instance> {
    let required_extensions = get_required_extensions(entry, display_handle)?;
    let app_info = get_app_info();
    let create_info = get_create_info(&required_extensions, &app_info);
    unsafe { entry.create_instance(&create_info, None)? }.pipe(Ok)
}

fn get_required_extensions(
    entry: &ash::Entry,
    display_handle: RawDisplayHandle,
) -> Result<Vec<*const c_char>> {
    let available_extensions = get_set_of_available_extensions(entry)?;
    let mut result = REQUIRED_EXTENSIONS
        .iter()
        .map(|extension| {
            if !available_extensions.contains(extension.to_str()?) {
                Err(ExtensionNotFound::new(extension))?;
            }
            Ok(extension.as_ptr())
        })
        .collect::<Result<Vec<*const c_char>>>()?;
    result.extend(ash_window::enumerate_required_extensions(display_handle)?);
    Ok(result)
}

fn get_set_of_available_extensions(entry: &ash::Entry) -> Result<HashSet<ExtensionName>> {
    unsafe { entry.enumerate_instance_extension_properties(None)? }
        .into_iter()
        .map(|elem| Ok(elem.extension_name_as_c_str()?.to_str()?.to_string()))
        .collect()
}

fn get_app_info() -> vk::ApplicationInfo<'static> {
    vk::ApplicationInfo::default()
        .api_version(vk::API_VERSION_1_3)
        .application_name(ENGINE_NAME_CSTR)
        .application_version(ENGINE_VERSION)
        .engine_name(ENGINE_NAME_CSTR)
        .engine_version(ENGINE_VERSION)
}

fn get_create_info<'a>(
    required_extensions: &'a [*const c_char],
    app_info: &'a vk::ApplicationInfo,
) -> vk::InstanceCreateInfo<'a> {
    let create_info = vk::InstanceCreateInfo::default()
        .application_info(app_info)
        .flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
        .enabled_extension_names(required_extensions);
    #[cfg(feature = "validation_layers")]
    {
        return create_info.enabled_layer_names(VALIDATION_LAYERS);
    }
    #[cfg(not(feature = "validation_layers"))]
    {
        create_info
    }
}
