use std::fs::File;
use std::io::Read;
use ash::vk;
use crate::utils::{PipeLine, Result};
use crate::vulkan::errors::{FailedToReadShaderCode, ShaderCodeBadLen};

pub const VERT_SHADER_PATH: &str = "./shaders/build/shader.vert.spv";
pub const FRAG_SHADER_PATH: &str = "./shaders/build/shader.frag.spv";

struct ShaderCode(Vec<u8>);

pub struct ShaderModule<'a> {
    module: vk::ShaderModule,
    device: &'a ash::Device,
}

impl Drop for ShaderModule<'_> {
    fn drop(&mut self) {
        unsafe { self.device.destroy_shader_module(self.module, None) }
    }
}

impl<'a> ShaderModule<'a> {
    pub fn new(device: &'a ash::Device,
               shader_binary_path: &'static str)
               -> Result<Self> {
        ShaderCode::new(shader_binary_path)?
            .pipe(|code| {
                shader_module_create_info(&code)
                    .pipe(|create_info| unsafe { device.create_shader_module(&create_info, None) })
            })?
            .pipe(|module| ShaderModule { module, device })
            .pipe(Ok)
    }
}


impl ShaderCode {
    fn new(shader_file_path: &'static str) -> Result<Self> {
        let mut u8_data = Vec::new();
        File::open(shader_file_path)
            .map_err(|error| FailedToReadShaderCode::new(shader_file_path, error.into()))?
            .read_to_end(&mut u8_data)
            .map_err(|error| FailedToReadShaderCode::new(shader_file_path, error.into()))?;

        if u8_data.len() % 4 != 0 {
            return Err(ShaderCodeBadLen::new(shader_file_path).into());
        }

        Ok(Self(u8_data))
    }

    fn as_u32_slice(&self) -> &[u32] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.as_ptr() as *const u32,
                self.0.len() / 4,
            )
        }
    }
}

fn shader_module_create_info(code: &ShaderCode) -> vk::ShaderModuleCreateInfo {
    vk::ShaderModuleCreateInfo::default()
        .code(code.as_u32_slice())
}
