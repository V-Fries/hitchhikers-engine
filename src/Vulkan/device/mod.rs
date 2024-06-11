#[allow(clippy::module_inception)]
mod device;
mod physical_device;

pub use device::create_device;
