use rs42::error_struct_custom_display;

error_struct_custom_display!(FailedToReadShaderCode {
    shader_file_path: &'static str,
    error: Box<dyn std::error::Error>,
}, "Failed to read shader \"{}\": {}", shader_file_path, error);

error_struct_custom_display!(ShaderCodeBadLen {
    shader_file_path: &'static str,
}, "The number of bytes in the \"{}\" shader is not a multiple of 4", shader_file_path);

error_struct_custom_display!(FailedToCreatePipeline {
    error: (Vec<ash::vk::Pipeline>, ash::vk::Result),
}, "Failed to create pipeline: {:?}", error);

error_struct_custom_display!(
    FailedToFindSupportedFormatForDepthBuffer,
    "No supported format for depth buffer",
);
