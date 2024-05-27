use anyhow::Result;

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
    pub fn new(extension_name: &'static CStr) -> Result<Self> {
        Err(Self {
            extension_name,
        })?
    }
}
