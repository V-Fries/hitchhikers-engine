use super::errors::ExtensionNotFound;
use super::validation_layers::VALIDATION_LAYERS;
use super::validation_layers::{check_validation_layers, create_debug_messenger};
use crate::engine::{ENGINE_NAME_CSTR, ENGINE_VERSION};
use crate::utils::{Defer, Result, ScopeGuard};
use ash::vk;

use std::collections::HashSet;
use std::ffi::{c_char, CStr, CString};
use winit::raw_window_handle::RawDisplayHandle;

type ExtensionName = CString;

const REQUIRED_EXTENSIONS: &[&CStr] = &[
    vk::KHR_PORTABILITY_ENUMERATION_NAME,
    #[cfg(feature = "validation_layers")]
    vk::EXT_DEBUG_UTILS_NAME,
];

pub fn create_instance(
    entry: &ash::Entry,
    display_handle: RawDisplayHandle,
) -> Result<(ash::Instance, Option<vk::DebugUtilsMessengerEXT>)> {
    if cfg!(feature = "validation_layers") {
        check_validation_layers(entry)?;
    }

    let required_extensions = get_required_extensions(entry, display_handle)?;
    let app_info = get_app_info();
    let create_info = get_create_info(&required_extensions, &app_info);
    let instance = unsafe { entry.create_instance(&create_info, None)? }
        .defer(|instance| unsafe { instance.destroy_instance(None) });

    if cfg!(feature = "validation_layers") {
        let debug_messenger = create_debug_messenger(entry, &instance)?;
        return Ok((ScopeGuard::into_inner(instance), Some(debug_messenger)));
    }
    Ok((ScopeGuard::into_inner(instance), None))
}

fn get_required_extensions(
    entry: &ash::Entry,
    display_handle: RawDisplayHandle,
) -> Result<Vec<*const c_char>> {
    let mut required_extensions = REQUIRED_EXTENSIONS
        .iter()
        .map(|elem| elem.as_ptr())
        .collect::<Vec<*const c_char>>();
    required_extensions.extend(ash_window::enumerate_required_extensions(display_handle)?);

    check_extensions_support(entry, &required_extensions)?;
    Ok(required_extensions)
}

fn check_extensions_support(
    entry: &ash::Entry,
    required_extensions: &[*const c_char],
) -> Result<()> {
    let available_extensions = get_set_of_available_extensions(entry)?;
    for extension in required_extensions.iter() {
        let extension = unsafe { CStr::from_ptr(*extension) };
        if !available_extensions.contains(extension) {
            Err(ExtensionNotFound::new(extension))?;
        }
    }
    Ok(())
}

fn get_set_of_available_extensions(entry: &ash::Entry) -> Result<HashSet<ExtensionName>> {
    unsafe { entry.enumerate_instance_extension_properties(None)? }
        .into_iter()
        .map(|elem| Ok(elem.extension_name_as_c_str()?.into()))
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
    if cfg!(feature = "validation_layers") {
        return create_info.enabled_layer_names(VALIDATION_LAYERS);
    }
    create_info
}
