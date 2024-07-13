use crate::error_struct;

error_struct!(FailedToCreateWindow {
    err: winit::error::OsError,
}, "Failed to create window: {}", err);

error_struct!(FailedToInitVulkan {
    err: Box<dyn std::error::Error>,
}, "Failed to init vulkan: {}", err);
