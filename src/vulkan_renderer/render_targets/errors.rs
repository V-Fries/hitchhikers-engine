use crate::error_struct;

error_struct!(FailedToReadShaderCode {
    shader_file_path: &'static str,
    error: Box<dyn std::error::Error>,
}, "Failed to read shader \"{}\": {}", shader_file_path, error);

error_struct!(ShaderCodeBadLen {
    shader_file_path: &'static str,
}, "The number of bytes in the \"{}\" shader is not a multiple of 4", shader_file_path);

error_struct!(FailedToCreatePipeline {
    error: (Vec<ash::vk::Pipeline>, ash::vk::Result),
}, "Failed to create pipeline: {:?}", error);
