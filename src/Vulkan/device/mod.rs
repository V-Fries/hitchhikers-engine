#[allow(clippy::module_inception)]
mod device;
mod physical_device;

pub use device::create_device;
pub use device::create_device_queue;
pub use device::Queues;
pub use physical_device::QueueFamilies;
pub use physical_device::DeviceData;
pub use physical_device::pick_physical_device;
