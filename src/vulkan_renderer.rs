mod vulkan_context;
mod vulkan_interface;
mod render_targets;

use crate::utils::Result;
use ash::{prelude::VkResult, vk};
use render_targets::RenderTargets;
use vulkan_context::VulkanContext;
use vulkan_interface::VulkanInterface;

const NB_OF_FRAMES_IN_FLIGHT: u32 = 2;
const NB_OF_FRAMES_IN_FLIGHT_USIZE: usize = NB_OF_FRAMES_IN_FLIGHT as usize;

pub struct VulkanRenderer {
    context: VulkanContext,
    interface: VulkanInterface,
    render_targets: RenderTargets,

    current_frame: usize,
}

impl VulkanRenderer {
    pub fn new(window: &winit::window::Window) -> Result<Self> {
        // TODO use a builder to handle leaks on error
        // TODO use generics instead of option in the builders
        let (context, queue_families, swapchain_builder) = VulkanContext::new(window)?;
        unsafe {
            Ok(Self {
                interface: VulkanInterface::new(&context, queue_families)?,
                render_targets: RenderTargets::new(&context, swapchain_builder)?,
                context,
                current_frame: 0,
            })
        }
    }

    pub fn render_frame(&mut self) -> VkResult<()> {
        self.wait_for_in_flight_fence()?;

        let image_index = self.acquire_next_image()?;

        self.reset_command_buffer()?;
        self.record_command_buffer(image_index)?;

        self.submit_command_buffer()?;
        self.present_image(image_index)?;

        self.current_frame = (self.current_frame + 1) % NB_OF_FRAMES_IN_FLIGHT_USIZE;
        Ok(())
    } 

    pub fn wait_for_in_flight_fence(&self) -> VkResult<()> {
        let fences = [self.interface.sync_objects().in_flight_fences[self.current_frame]];
        unsafe {
            self.context.device().wait_for_fences(&fences, true, u64::MAX)?;
            self.context.device().reset_fences(&fences)?;
        }
        Ok(())
    }

    pub fn acquire_next_image(&self) -> VkResult<u32> {
        let index = unsafe {
            self.render_targets.swapchain_device().acquire_next_image(
                self.render_targets.swapchain(),
                u64::MAX,
                self.interface.sync_objects().image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )?.0
        };
        Ok(index)
    }

    pub fn reset_command_buffer(&self) -> VkResult<()> {
        unsafe {
            self.context.device().reset_command_buffer(self.interface.command_buffers()[self.current_frame],
                                                  vk::CommandBufferResetFlags::empty())
        }
    }

    pub fn record_command_buffer(&self, image_index: u32) -> VkResult<()> {
        // TODO refactor
        let begin_info = vk::CommandBufferBeginInfo::default();
        let command_buffer = self.interface.command_buffers()[self.current_frame];

        unsafe {
            self.context.device().begin_command_buffer(command_buffer, &begin_info)?;
        }

        let clear_values = [
            vk::ClearValue {
                // TODO check if I should use float32
                color: vk::ClearColorValue { uint32: [0, 0, 0, 1] }
            }
        ];
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_targets.render_pass())
            .framebuffer(self.render_targets.framebuffers()[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.render_targets.swapchain_extent(),
            })
            .clear_values(&clear_values);

        unsafe {
            self.context.device().cmd_begin_render_pass(
                command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE,
            );

            self.context.device().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.render_targets.pipeline(),
            );
        }

        let viewports = [vk::Viewport::default()
            .x(0.)
            .y(0.)
            .width(self.render_targets.swapchain_extent().width as f32)
            .height(self.render_targets.swapchain_extent().height as f32)
            .min_depth(0.)
            .max_depth(1.)];
        let scissors = [vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(self.render_targets.swapchain_extent())];
        unsafe {
            self.context.device().cmd_set_viewport(command_buffer, 0, &viewports);
            self.context.device().cmd_set_scissor(command_buffer, 0, &scissors);
            self.context.device().cmd_draw(command_buffer, 3, 1, 0, 0);
            self.context.device().cmd_end_render_pass(command_buffer);
            self.context.device().end_command_buffer(command_buffer)?;
        }
        Ok(())
    }

    pub fn submit_command_buffer(&self) -> VkResult<()> {
        let wait_semaphores = [self.interface.sync_objects().image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.interface.command_buffers()[self.current_frame]];
        let signal_semaphores = [self.interface.sync_objects().render_finished_semaphores[self.current_frame]];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.context.device().queue_submit(
                self.interface.queues().graphics_queue(), &[submit_info],
                self.interface.sync_objects().in_flight_fences[self.current_frame],
            )
        }
    }

    pub fn present_image(&self, image_index: u32) -> VkResult<()> {
        let wait_semaphores = [
            self.interface.sync_objects().render_finished_semaphores[self.current_frame]
        ];
        let swapchains = [self.render_targets.swapchain()];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.render_targets.swapchain_device().queue_present(
                self.interface.queues().present_queue(), &present_info,
            ).map(|_| ())
        }
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            if let Err(err) = self.context.device().device_wait_idle() {
                eprintln!("Failed to wait for vulkan device to be idle: {err}.
                           Attempting to clean resources anyway...");
            }

            self.interface.destroy(&self.context);
            self.render_targets.destroy(&self.context);
            self.context.destroy();
        }
    }
}

