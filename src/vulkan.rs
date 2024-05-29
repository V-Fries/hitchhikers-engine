mod errors;

#[cfg(feature = "validation_layers")]
mod validation_layers;

#[cfg(feature = "validation_layers")]
use validation_layers::*;
use errors::ExtensionNotFound;

use ash::vk;
use anyhow::Result;
use ash::vk::ApplicationInfo;

use std::collections::HashSet;
use std::ffi::{c_char, CStr};


const ENGINE_NAME: &CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"HitchHiker's Engine\0") };
const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);

const REQUIRED_EXTENSIONS: &[&CStr] = &[
    vk::KHR_PORTABILITY_ENUMERATION_NAME,
    #[cfg(feature = "validation_layers")]
        vk::EXT_DEBUG_UTILS_NAME,
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
    fn drop(&mut self) {
        unsafe {
            #[cfg(feature = "validation_layers")] {
                ash::ext::debug_utils::Instance::new(&self.entry, &self.instance)
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
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
