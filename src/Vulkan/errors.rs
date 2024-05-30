use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};

pub struct ExtensionNotFound {
    extension_name: &'static CStr,
}

impl Debug for ExtensionNotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not find extension: {:?}", self.extension_name)
    }
}

impl Display for ExtensionNotFound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for ExtensionNotFound {}

impl ExtensionNotFound {
    pub fn new(extension_name: &'static CStr) -> Self {
        Self {
            extension_name,
        }
    }
}


pub struct NoSuitablePhysicalDevice;

impl Debug for NoSuitablePhysicalDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not find any suitable physical device")
    }
}

impl Display for NoSuitablePhysicalDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for NoSuitablePhysicalDevice {}


#[cfg(feature = "validation_layers")]
pub mod validation_layers {
    use super::*;

    pub struct ValidationLayerNotFound {
        validation_layer_name: &'static CStr,
    }

    impl Debug for ValidationLayerNotFound {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Could not find validation layer: {:?}",
                   self.validation_layer_name)
        }
    }

    impl Display for ValidationLayerNotFound {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{self:?}")
        }
    }

    impl Error for ValidationLayerNotFound {}

    impl ValidationLayerNotFound {
        pub fn new(validation_layer_name: &'static CStr) -> Self {
            Self {
                validation_layer_name,
            }
        }
    }
}
