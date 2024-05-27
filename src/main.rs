mod vulkan;

use vulkan::Vulkan;

fn main() -> anyhow::Result<()> {
    let vulkan_instance = Vulkan::new()?;
    Ok(())
}
