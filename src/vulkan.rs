mod extension_not_found;

use ash::vk;
use std::collections::HashSet;
use std::ffi::{c_char, CStr};
use anyhow::Result;
use ash::vk::ApplicationInfo;
use extension_not_found::ExtensionNotFound;

const ENGINE_NAME: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"HitchHiker's Engine\0") };
const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 1);

pub struct Vulkan {
    entry: ash::Entry,
    instance: ash::Instance,
}

impl Vulkan {
    pub fn new() -> Result<Self> {
        let entry = unsafe { ash::Entry::load()? };

        let available_extensions = get_set_of_available_extensions(&entry)?;

        let required_extensions = get_required_extensions(available_extensions)?;

        let app_info = get_app_info();

        let create_info = get_create_info(&required_extensions, &app_info);

        let instance = unsafe { entry.create_instance(&create_info, None)? };

        Ok(Self { entry, instance })
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}

fn get_set_of_available_extensions(entry: &ash::Entry) -> Result<HashSet<String>> {
    unsafe { entry.enumerate_instance_extension_properties(None)? }
        .into_iter()
        .map::<Result<String>, _>(|elem| {
            Ok(unsafe { CStr::from_ptr(elem.extension_name.as_ptr()) }
                .to_str()?
                .to_string())
        })
        .collect()
}

fn get_required_extensions(available_extensions: HashSet<String>) -> Result<Vec<*const c_char>> {
    let mut result = Vec::new();

    if !available_extensions.contains(vk::KHR_PORTABILITY_ENUMERATION_NAME.to_str()?) {
        ExtensionNotFound::new(vk::KHR_PORTABILITY_ENUMERATION_NAME)?;
    }
    result.push(vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr());

    Ok(result)
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
    vk::InstanceCreateInfo::default()
        .application_info(app_info)
        .flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
        .enabled_extension_names(required_extensions)
}
