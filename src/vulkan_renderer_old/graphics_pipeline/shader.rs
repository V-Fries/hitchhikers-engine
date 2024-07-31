use std::fs::File;
use std::io::Read;
use ash::vk;
use crate::utils::{PipeLine, Result};
use crate::vulkan_renderer_old::errors::{FailedToReadShaderCode, ShaderCodeBadLen};

const VERT_SHADER_PATH: &str = "./shaders/build/shader.vert.spv";
const FRAG_SHADER_PATH: &str = "./shaders/build/shader.frag.spv";

pub struct ShaderStageCreateInfos<'a> {
    create_infos: [vk::PipelineShaderStageCreateInfo<'a>; 2],
    #[allow(dead_code)]
    vertex_shader_module: ShaderModule<'a>,
    #[allow(dead_code)]
    fragment_shader_module: ShaderModule<'a>,
}

struct ShaderModule<'a> {
    module: vk::ShaderModule,
    device: &'a ash::Device,
}

struct ShaderCode(Vec<u8>);

impl<'a> ShaderStageCreateInfos<'a> {
    pub fn new(device: &'a ash::Device) -> Result<Self> {
        let vertex_shader_module = ShaderModule::new(device, VERT_SHADER_PATH)?;
        let fragment_shader_module = ShaderModule::new(device, FRAG_SHADER_PATH)?;

        let vertex_shader_stage_create_info =
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader_module.module())
                .name(c"main");
        let fragment_shader_stage_create_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader_module.module())
            .name(c"main");

        let create_infos = [
            vertex_shader_stage_create_info,
            fragment_shader_stage_create_info,
        ];

        Ok(Self { create_infos, vertex_shader_module, fragment_shader_module })
    }

    pub fn create_infos(&self) -> &[vk::PipelineShaderStageCreateInfo] {
        &self.create_infos
    }
}


impl<'a> ShaderModule<'a> {
    fn new(device: &'a ash::Device,
           shader_binary_path: &'static str)
           -> Result<Self> {
        ShaderCode::new(shader_binary_path)?
            .pipe(|code| {
                Self::shader_module_create_info(&code)
                    .pipe(|create_info| unsafe { device.create_shader_module(&create_info, None) })
            })?
            .pipe(|module| ShaderModule { module, device })
            .pipe(Ok)
    }

    fn module(&self) -> vk::ShaderModule {
        self.module
    }

    fn shader_module_create_info(code: &ShaderCode) -> vk::ShaderModuleCreateInfo {
        vk::ShaderModuleCreateInfo::default()
            .code(code.as_u32_slice())
    }
}


impl Drop for ShaderModule<'_> {
    fn drop(&mut self) {
        unsafe { self.device.destroy_shader_module(self.module, None) }
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
