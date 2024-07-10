macro_rules! error_struct {
    ($struct_name:ident $( { $( $field_name:ident : $field_type:ty ),* $(,)? } )?,
     $format_message:expr $(, $( $format_var:ident ), * $(,)? )?) => {
        pub struct $struct_name {
            $( $( $field_name : $field_type ),*, )?
        }

        impl $struct_name {
            #[allow(dead_code)]
            pub fn new($( $( $field_name : $field_type ),* )?) -> Self {
                $struct_name {
                    $( $( $field_name ),* )?
                }
            }
        }

        impl std::fmt::Display for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $format_message $(, $( self.$format_var ), * )?)
            }
        }

        impl std::fmt::Debug for $struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self}")
            }
        }

        impl std::error::Error for $struct_name {}
    };
}

error_struct!(ExtensionNotFound {
    extension_name: &'static std::ffi::CStr,
}, "Could not find extension: {:?}", extension_name);

error_struct!(NoSuitablePhysicalDevice,
              "Could not find any suitable physical device");

error_struct!(PhysicalDeviceIsNotSuitable {
    device: ash::vk::PhysicalDevice,
    reason: String,
}, "Physical device {:?} is not suitable: {}", device, reason);

#[cfg(feature = "validation_layers")]
error_struct!(ValidationLayerNotFound {
    validation_layer_name: &'static std::ffi::CStr,
}, "Could not find validation layer: {:?}", validation_layer_name);

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