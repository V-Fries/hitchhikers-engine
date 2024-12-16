use rs42::error_struct_custom_display;

error_struct_custom_display!(
    FailedToCreateWindow {
        err: winit::error::OsError,
    },
    "Failed to create window: {}",
    err
);

error_struct_custom_display!(FailedToInitVulkan {
    err: Box<dyn std::error::Error>,
}, "Failed to init vulkan: {}", err);
