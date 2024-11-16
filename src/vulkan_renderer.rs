mod buffer;
mod memory;
mod render_targets;
mod single_time_command;
mod uniform_buffer_object;
mod vulkan_context;
mod vulkan_interface;

use std::{ptr::copy_nonoverlapping, time::SystemTime};

use crate::utils::{Defer, Result, ScopeGuard};
use ash::{prelude::VkResult, vk};
use linear_algebra::{Degree, Matrix};
use memory::{Memory, INDICES};
use render_targets::RenderTargets;
use uniform_buffer_object::UniformBufferObject;
use vulkan_context::{create_device, PhysicalDeviceData, SwapchainBuilder, VulkanContext};
use vulkan_interface::VulkanInterface;

const NB_OF_FRAMES_IN_FLIGHT: u32 = 2;
const NB_OF_FRAMES_IN_FLIGHT_USIZE: usize = NB_OF_FRAMES_IN_FLIGHT as usize;

pub struct VulkanRenderer {
    context: VulkanContext,
    interface: VulkanInterface,
    render_targets: RenderTargets,
    memory: Memory,

    previous_frame_start_time: SystemTime,

    current_frame: usize,

    rotation: Degree<f32>,
}

type ShouldStopRenderingFrame = bool;

enum NextImage {
    Index(u32),
    ShouldStopRenderingFrame,
}

impl VulkanRenderer {
    pub fn new(window: &winit::window::Window) -> Result<Self> {
        let (context, queue_families, swapchain_builder) = VulkanContext::new(window)?;
        let context = context.defer(|mut context| unsafe { context.destroy() });

        let interface = unsafe { VulkanInterface::new(&context, queue_families)? }
            .defer(|mut interface| unsafe { interface.destroy(context.device()) });

        let render_targets = unsafe { RenderTargets::new(&context, swapchain_builder)? }
            .defer(|mut render_targets| unsafe { render_targets.destroy(&context) });

        let memory = unsafe { Memory::new(&context, &interface, &render_targets)? }
            .defer(|mut memory| unsafe { memory.destroy(context.device()) });

        Ok(Self {
            rotation: Degree::from(90.),
            current_frame: 0,
            previous_frame_start_time: SystemTime::now(),
            memory: ScopeGuard::into_inner(memory),
            render_targets: ScopeGuard::into_inner(render_targets),
            interface: ScopeGuard::into_inner(interface),
            context: ScopeGuard::into_inner(context),
        })
    }

    pub fn render_frame(&mut self, window: &winit::window::Window) -> Result<()> {
        self.wait_for_in_flight_fence()?;

        let NextImage::Index(image_index) = self.acquire_next_image(window)? else {
            return Ok(());
        };

        self.update_uniform_buffer();

        self.reset_in_flight_fence()?;

        self.reset_command_buffer()?;
        unsafe { self.record_command_buffer(image_index)? }

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

    fn update_uniform_buffer(&mut self) {
        let current_time = SystemTime::now();
        let elapsed_time_sec = match current_time.duration_since(self.previous_frame_start_time) {
            Ok(elapsed_time) => elapsed_time.as_secs_f32(),
            Err(_) => 0., // TODO find a cleaner way to handle this
        };
        self.previous_frame_start_time = current_time;

        *self.rotation += 90. * elapsed_time_sec;

        let aspect_ratio = self.render_targets.swapchain_extent().width as f32
            / self.render_targets.swapchain_extent().height as f32;
        let mut uniform_buffer_object = UniformBufferObject {
            model: Matrix::model(
                [0., 0., 1.],
                self.rotation.clone(),
                [0., 0., 0.],
                [1., 1., 1.],
            ),
            view: Matrix::look_at([2., 2., 2.], [0., 0., 0.], [0., 0., 1.]),
            proj: Matrix::perspective_opengl(Degree::from(45.), aspect_ratio, 0.1, 10.),
        };
        uniform_buffer_object.proj[1][1] *= -1.;

        unsafe {
            copy_nonoverlapping(
                &uniform_buffer_object,
                self.memory.mapped_uniform_buffers()[self.current_frame]
                    as *mut UniformBufferObject,
                1,
            )
        };
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

    unsafe fn record_command_buffer(&self, image_index: u32) -> VkResult<()> {
        // TODO refactor
        let begin_info = vk::CommandBufferBeginInfo::default();
        let command_buffer = self.interface.command_buffers()[self.current_frame];

        self.context
            .device()
            .begin_command_buffer(command_buffer, &begin_info)?;

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

        let vertex_buffers = [self.memory.vertex_buffer().buffer()];
        let offsets = [0];
        self.context
            .device()
            .cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
        self.context.device().cmd_bind_index_buffer(
            command_buffer,
            self.memory.index_buffer().buffer(),
            0,
            vk::IndexType::UINT16,
        );

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
        self.context
            .device()
            .cmd_set_viewport(command_buffer, 0, &viewports);
        self.context
            .device()
            .cmd_set_scissor(command_buffer, 0, &scissors);
        self.context.device().cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.render_targets.pipeline_layout(),
            0,
            &[self.memory.descriptor_sets()[self.current_frame]],
            &[],
        );
        self.context
            .device()
            .cmd_draw_indexed(command_buffer, INDICES.len() as u32, 1, 0, 0, 0);
        self.context.device().cmd_end_render_pass(command_buffer);
        self.context.device().end_command_buffer(command_buffer)?;
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

        self.interface.destroy(self.context.device());
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
            physical_device_data.physical_device_properties,
            physical_device_data.physical_device_features,
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
        self.memory.destroy(self.context.device());
        self.interface.destroy(self.context.device());
        self.render_targets.destroy(&self.context);
        self.context.destroy();
    }
}
