mod buffer;
mod builder;
mod render_targets;
mod vulkan_context;
mod vulkan_interface;

use crate::utils::{PipeLine, Result};
use ash::{prelude::VkResult, vk};
use buffer::Buffer;
use builder::VulkanRendererBuilder;
use render_targets::RenderTargets;
use vulkan_context::{create_device, PhysicalDeviceData, SwapchainBuilder, VulkanContext};
use vulkan_interface::VulkanInterface;

const NB_OF_FRAMES_IN_FLIGHT: u32 = 2;
const NB_OF_FRAMES_IN_FLIGHT_USIZE: usize = NB_OF_FRAMES_IN_FLIGHT as usize;

pub struct VulkanRenderer {
    context: VulkanContext,
    interface: VulkanInterface,
    render_targets: RenderTargets,
    vertex_buffer: Buffer,

    current_frame: usize,
}

type ShouldStopRenderingFrame = bool;

enum NextImage {
    Index(u32),
    ShouldStopRenderingFrame,
}

impl VulkanRenderer {
    pub fn new(window: &winit::window::Window) -> Result<Self> {
        VulkanRendererBuilder::default()
            .create_context(window)?
            .create_interface()?
            .create_render_targets()?
            .create_vertex_buffer()?
            .build()
            .pipe(Ok)
    }

    pub fn render_frame(&mut self, window: &winit::window::Window) -> Result<()> {
        self.wait_for_in_flight_fence()?;

        let NextImage::Index(image_index) = self.acquire_next_image(window)? else {
            return Ok(());
        };

        self.reset_in_flight_fence()?;

        self.reset_command_buffer()?;
        self.record_command_buffer(image_index)?;

        self.submit_command_buffer()?;
        if self.present_image(image_index, window)? {
            return Ok(());
        }

        self.current_frame = (self.current_frame + 1) % NB_OF_FRAMES_IN_FLIGHT_USIZE;
        Ok(())
    }

    fn wait_for_in_flight_fence(&self) -> VkResult<()> {
        unsafe {
            self.context.device().wait_for_fences(
                &[self.interface.sync_objects().in_flight_fences[self.current_frame]],
                true,
                u64::MAX,
            )
        }
    }

    fn reset_in_flight_fence(&self) -> VkResult<()> {
        unsafe {
            self.context
                .device()
                .reset_fences(&[self.interface.sync_objects().in_flight_fences[self.current_frame]])
        }
    }

    fn acquire_next_image(&mut self, window: &winit::window::Window) -> Result<NextImage> {
        match unsafe {
            self.render_targets.swapchain_device().acquire_next_image(
                self.render_targets.swapchain(),
                u64::MAX,
                self.interface.sync_objects().image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )
        } {
            Ok((index, is_suboptimal)) => {
                if !is_suboptimal {
                    return Ok(NextImage::Index(index));
                }
            }
            Err(err) => {
                if err != vk::Result::ERROR_OUT_OF_DATE_KHR {
                    return Err(err.into());
                }
            }
        };
        // TODO the image available semaphore is left signaled and unused!!
        //      to test this easily, disable Resized event handling
        unsafe {
            self.recreate_swapchain(window)?;
        }
        Ok(NextImage::ShouldStopRenderingFrame)
    }

    fn reset_command_buffer(&self) -> VkResult<()> {
        unsafe {
            self.context.device().reset_command_buffer(
                self.interface.command_buffers()[self.current_frame],
                vk::CommandBufferResetFlags::empty(),
            )
        }
    }

    fn record_command_buffer(&self, image_index: u32) -> VkResult<()> {
        // TODO refactor
        let begin_info = vk::CommandBufferBeginInfo::default();
        let command_buffer = self.interface.command_buffers()[self.current_frame];

        unsafe {
            self.context
                .device()
                .begin_command_buffer(command_buffer, &begin_info)?;
        }

        let clear_values = [vk::ClearValue {
            // TODO check if I should use float32
            color: vk::ClearColorValue {
                uint32: [0, 0, 0, 1],
            },
        }];
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
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            self.context.device().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.render_targets.pipeline(),
            );
        }

        let vertex_buffers = [self.vertex_buffer.buffer()];
        let offsets = [0];
        unsafe {
            self.context.device().cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &vertex_buffers,
                &offsets,
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
            self.context
                .device()
                .cmd_set_viewport(command_buffer, 0, &viewports);
            self.context
                .device()
                .cmd_set_scissor(command_buffer, 0, &scissors);
            self.context.device().cmd_draw(command_buffer, 3, 1, 0, 0); // TODO remove the hardcoded 3
            self.context.device().cmd_end_render_pass(command_buffer);
            self.context.device().end_command_buffer(command_buffer)?;
        }
        Ok(())
    }

    fn submit_command_buffer(&self) -> VkResult<()> {
        let wait_semaphores =
            [self.interface.sync_objects().image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.interface.command_buffers()[self.current_frame]];
        let signal_semaphores =
            [self.interface.sync_objects().render_finished_semaphores[self.current_frame]];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.context.device().queue_submit(
                self.interface.queues().graphics_queue(),
                &[submit_info],
                self.interface.sync_objects().in_flight_fences[self.current_frame],
            )
        }
    }

    fn present_image(
        &mut self,
        image_index: u32,
        window: &winit::window::Window,
    ) -> Result<ShouldStopRenderingFrame> {
        let wait_semaphores =
            [self.interface.sync_objects().render_finished_semaphores[self.current_frame]];
        let swapchains = [self.render_targets.swapchain()];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        match unsafe {
            self.render_targets
                .swapchain_device()
                .queue_present(self.interface.queues().present_queue(), &present_info)
        } {
            Ok(is_suboptimal) => {
                if !is_suboptimal {
                    return Ok(false);
                }
            }
            Err(err) => {
                if err != vk::Result::ERROR_OUT_OF_DATE_KHR {
                    return Err(err.into());
                }
            }
        };
        unsafe {
            self.recreate_swapchain(window)?;
        }
        Ok(true)
    }

    pub unsafe fn recreate_swapchain(&mut self, window: &winit::window::Window) -> Result<()> {
        if let Err(err) = unsafe { self.context.device().device_wait_idle() } {
            eprintln!("Swapchain could not be updated: Failed to wait for vulkan device to be idle: {err}.");
            return Ok(());
        }

        self.render_targets.destroy(&self.context);

        let swapchain_builder = self.create_swapchain_builder(window)?;

        // TODO this can be optimized as the render pass doesn't always
        //      need to be recreated
        // TODO do research about the following statement:
        //      It is possible to create a new swap chain while drawing
        //      commands on an image from the old swap chain are still
        //      in-flight. You need to pass the previous swap chain to the
        //      oldSwapChain field in the VkSwapchainCreateInfoKHR struct
        //      and destroy the old swap chain as soon as you've finished
        //      using it.

        self.render_targets = RenderTargets::new(&self.context, swapchain_builder)?;
        Ok(())
    }

    pub unsafe fn create_swapchain_builder(
        &mut self,
        window: &winit::window::Window,
    ) -> Result<SwapchainBuilder> {
        let window_inner_size = window.inner_size();

        if let Ok(swapchain_builder) = SwapchainBuilder::new(
            self.context.physical_device(),
            self.interface.queue_families(),
            self.context.surface_instance(),
            self.context.surface(),
            window_inner_size,
        ) {
            return Ok(swapchain_builder);
        }

        self.interface.destroy(&self.context);
        self.context.destroy_device();

        let physical_device_data = PhysicalDeviceData::new(
            self.context.surface_instance(),
            self.context.instance(),
            self.context.surface(),
            window_inner_size,
        )?;

        self.context.set_device(
            create_device(self.context.instance(), &physical_device_data)?,
            physical_device_data.physical_device,
        );

        self.interface = VulkanInterface::new(&self.context, physical_device_data.queue_families)?;

        Ok(physical_device_data.swapchain_builder)
    }

    pub unsafe fn destroy(&mut self) {
        if let Err(err) = self.context.device_wait_idle() {
            eprintln!(
                "Failed to wait for vulkan device to be idle: {err}.
                       Attempting to clean resources anyway..."
            );
        }
        self.vertex_buffer.destroy(self.context.device());
        self.interface.destroy(&self.context);
        self.render_targets.destroy(&self.context);
        self.context.destroy();
    }
}
