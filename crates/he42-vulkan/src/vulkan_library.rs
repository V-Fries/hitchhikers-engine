use rs42::extensions::PipeLine;
use std::{ops::Deref, sync::Arc};

pub use ash::LoadingError;

/// Represents the Vulkan library
/// Holds the Vulkan functions independent of a particular instance
pub struct VulkanLibrary(pub(crate) ash::Entry);

impl VulkanLibrary {
    /// Load default Vulkan library for the current platform
    ///
    /// # Safety
    ///
    /// `dlopen`ing native libraries is inherently unsafe. The safety guidelines
    /// for [`Library::new()`] and [`Library::get()`] apply here.
    pub unsafe fn new() -> Result<Arc<Self>, LoadingError> {
        unsafe { ash::Entry::load() }?
            .pipe(VulkanLibrary)
            .pipe(Arc::new)
            .pipe(Ok)
    }
}

impl Deref for VulkanLibrary {
    type Target = ash::Entry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
