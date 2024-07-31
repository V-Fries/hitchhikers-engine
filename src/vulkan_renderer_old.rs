mod errors;
#[cfg(feature = "validation_layers")]
mod validation_layers;
mod instance;
mod device;
mod swapchain;
mod builder;
mod image_views;
mod graphics_pipeline;
mod sync_objects;

use ash::prelude::VkResult;
use ash::vk;

use crate::utils::{PipeLine, Result};
use crate::vulkan_renderer_old::builder::VulkanRendererBuilder;

pub struct VulkanRenderer {
    entry: ash::Entry,
    instance: ash::Instance,
    #[cfg(feature = "validation_layers")]
    debug_messenger: vk::DebugUtilsMessengerEXT,

    device: ash::Device,
    queues: device::Queues,

    surface: vk::SurfaceKHR,

    swapchain_device: ash::khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    swapchain_image_views: Vec<vk::ImageView>,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    framebuffers: Vec<vk::Framebuffer>,

    command_pool: vk::CommandPool,
    command_buffers: Box<[vk::CommandBuffer]>,

    image_available_semaphores: Box<[vk::Semaphore]>,
    render_finished_semaphores: Box<[vk::Semaphore]>,
    in_flight_fences: Box<[vk::Fence]>,

    current_frame: usize,
    nb_of_frames_in_flight: usize,
}

impl VulkanRenderer {
    pub fn new(window: &winit::window::Window,
               nb_of_frames_in_flight: u32)
               -> Result<Self> {
        VulkanRendererBuilder::new(window, nb_of_frames_in_flight)?
            .build()
            .pipe(Ok)
    }

    pub fn render_frame(&mut self) -> VkResult<()> {
        self.wait_for_in_flight_fence()?;

        let image_index = self.acquire_next_image()?;

        self.reset_command_buffer()?;
        self.record_command_buffer(image_index)?;

        self.submit_command_buffer()?;
        self.present_image(image_index)?;

        self.current_frame = (self.current_frame + 1) % self.nb_of_frames_in_flight;
        Ok(())
    }

    fn record_command_buffer(&self, image_index: u32) -> VkResult<()> {
        // TODO refactor
        let begin_info = vk::CommandBufferBeginInfo::default();
        let command_buffer = self.command_buffers[self.current_frame];

        unsafe {
            self.device.begin_command_buffer(command_buffer, &begin_info)?;
        }

        let clear_values = [
            vk::ClearValue {
                // TODO check if I should use float32
                color: vk::ClearColorValue { uint32: [0, 0, 0, 1] }
            }
        ];
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE,
            );

            self.device.cmd_bind_pipeline(
                command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline,
            );
        }

        let viewports = [vk::Viewport::default()
            .x(0.)
            .y(0.)
            .width(self.swapchain_extent.width as f32)
            .height(self.swapchain_extent.height as f32)
            .min_depth(0.)
            .max_depth(1.)];
        let scissors = [vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(self.swapchain_extent)];
        unsafe {
            self.device.cmd_set_viewport(command_buffer, 0, &viewports);
            self.device.cmd_set_scissor(command_buffer, 0, &scissors);
            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            self.device.cmd_end_render_pass(command_buffer);
            self.device.end_command_buffer(command_buffer)?;
        }
        Ok(())
    }

    fn wait_for_in_flight_fence(&self) -> VkResult<()> {
        let fences = [self.in_flight_fences[self.current_frame]];
        unsafe {
            self.device.wait_for_fences(&fences, true, u64::MAX)?;
            self.device.reset_fences(&fences)?;
        }
        Ok(())
    }

    fn acquire_next_image(&self) -> VkResult<u32> {
        let index = unsafe {
            self.swapchain_device.acquire_next_image(
                self.swapchain, u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )?.0
        };
        Ok(index)
    }

    fn reset_command_buffer(&self) -> VkResult<()> {
        unsafe {
            self.device.reset_command_buffer(self.command_buffers[self.current_frame],
                                             vk::CommandBufferResetFlags::empty())
        }
    }

    fn submit_command_buffer(&self) -> VkResult<()> {
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.command_buffers[self.current_frame]];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.device.queue_submit(
                self.queues.graphics_queue, &[submit_info],
                self.in_flight_fences[self.current_frame],
            )
        }
    }

    fn present_image(&self, image_index: u32) -> VkResult<()> {
        let wait_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let swapchains = [self.swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain_device.queue_present(
                self.queues.present_queue, &present_info,
            ).map(|_| ())
        }
    }

    fn update_swapchain(&mut self) -> VkResult<()> {
        println!("update_swapchain() called");
        unsafe { self.device.device_wait_idle()? };

        println!("update_swapchain() success");
        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            if let Err(err) = self.device.device_wait_idle() {
                eprintln!("Failed to wait for vulkan device to be idle: {err}.
                           Attempting to clean resources anyway...");
            }

            for semaphore in self.image_available_semaphores.iter() {
                self.device.destroy_semaphore(*semaphore, None);
            }
            for semaphore in self.render_finished_semaphores.iter() {
                self.device.destroy_semaphore(*semaphore, None);
            }
            for fence in self.in_flight_fences.iter() {
                self.device.destroy_fence(*fence, None);
            }
            self.device.destroy_command_pool(self.command_pool, None);
            for framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(*framebuffer, None);
            }
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(*image_view, None);
            }
            self.swapchain_device.destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            #[cfg(feature = "validation_layers")] {
                ash::ext::debug_utils::Instance::new(&self.entry, &self.instance)
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            ash::khr::surface::Instance::new(&self.entry, &self.instance)
                .destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
