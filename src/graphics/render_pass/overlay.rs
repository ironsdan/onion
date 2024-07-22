use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::{CommandBufferAllocator, StandardCommandBufferAllocator},
        CommandBuffer, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsage,
        RecordingCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        SubpassEndInfo,
    },
    device::Queue,
    format::Format,
    image::Image,
    memory::allocator::StandardMemoryAllocator,
    render_pass::{Framebuffer, RenderPass, Subpass},
    sync::GpuFuture,
    Validated, ValidationError, VulkanError,
};
use vulkano::{image::view::ImageView, render_pass::FramebufferCreateInfo};

pub struct RenderPassOverlay {
    pub gfx_queue: Arc<Queue>,
    pub render_pass: Arc<RenderPass>,
    pub cb_allocator: Arc<dyn CommandBufferAllocator>,
}

impl RenderPassOverlay {
    pub fn new(gfx_queue: Arc<Queue>, format: Format) -> Result<Self, Validated<VulkanError>> {
        let device = gfx_queue.device().clone();
        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: format,
                    samples: 1,
                    load_op: Load,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        )?;

        let cb_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Ok(Self {
            gfx_queue,
            render_pass,
            cb_allocator,
        })
    }

    pub fn cb_allocator(&self) -> Arc<dyn CommandBufferAllocator> {
        self.cb_allocator.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.gfx_queue.clone()
    }

    pub fn frame<F>(
        &mut self,
        before_future: F,
        final_image: Arc<Image>,
        _memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<OverlayFrame, Validated<VulkanError>>
    where
        F: GpuFuture + 'static,
    {
        let framebuffer = framebuffer_setup(final_image.clone(), self.render_pass.clone());

        let mut command_buffer = RecordingCommandBuffer::new(
            self.cb_allocator.clone(),
            self.gfx_queue.queue_family_index(),
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )?;
        command_buffer.begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![None],

                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            },
            SubpassBeginInfo {
                contents: SubpassContents::SecondaryCommandBuffers,
                ..Default::default()
            },
        )?;
        Ok(OverlayFrame {
            system: self,
            num_pass: 0,
            framebuffer,
            before_main_cb_future: Some(before_future.boxed()),
            command_buffer: Some(command_buffer),
        })
    }

    pub fn draw_pass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }
}

pub struct OverlayFrame<'a> {
    system: &'a mut RenderPassOverlay,
    num_pass: u8,
    framebuffer: Arc<Framebuffer>,
    before_main_cb_future: Option<Box<dyn GpuFuture>>,
    command_buffer: Option<RecordingCommandBuffer>,
}

impl<'a> OverlayFrame<'a> {
    pub fn next_pass<'f>(
        &'f mut self,
    ) -> Result<Option<OverlayPass<'f, 'a>>, Box<ValidationError>> {
        Ok(
            match {
                let current_pass = self.num_pass;
                self.num_pass += 1;
                current_pass
            } {
                0 => Some(OverlayPass::Draw(OverlayDrawPass { frame: self })),
                1 => {
                    // ToDo; Once you add more subpasses, remember to go to those...
                    // self.command_buffer_builder
                    //     .as_mut()
                    //     .unwrap()
                    //     .next_subpass(SubpassContents::SecondaryCommandBuffers)?;
                    self.command_buffer
                        .as_mut()
                        .unwrap()
                        .end_render_pass(SubpassEndInfo::default())?;
                    let command_buffer = self.command_buffer.take().unwrap().end().unwrap();

                    let after_main_cb = self
                        .before_main_cb_future
                        .take()
                        .unwrap()
                        .then_execute(self.system.gfx_queue.clone(), command_buffer)
                        .unwrap(); // TODO convert back to error type
                    Some(OverlayPass::Finished(after_main_cb.boxed()))
                }
                _ => None,
            },
        )
    }
}

/// Struct provided to the user that allows them to customize or handle the pass.
pub enum OverlayPass<'f, 's: 'f> {
    Draw(OverlayDrawPass<'f, 's>),
    Finished(Box<dyn GpuFuture>),
}

/// Allows the user to draw objects on the scene.
pub struct OverlayDrawPass<'f, 's: 'f> {
    frame: &'f mut OverlayFrame<'s>,
}

impl<'f, 's: 'f> OverlayDrawPass<'f, 's> {
    pub fn viewport_dimensions(&self) -> [u32; 2] {
        self.frame.framebuffer.extent()
    }

    /// Appends a command that executes a secondary command buffer that performs drawing.
    #[inline]
    pub fn execute(
        &mut self,
        command_buffer: Arc<CommandBuffer>,
    ) -> Result<(), Box<ValidationError>> {
        self.frame
            .command_buffer
            .as_mut()
            .unwrap()
            .execute_commands(command_buffer)?;
        Ok(())
    }
}

fn framebuffer_setup(image: Arc<Image>, render_pass: Arc<RenderPass>) -> Arc<Framebuffer> {
    let view = ImageView::new_default(image.clone()).unwrap();
    Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![view],
            ..Default::default()
        },
    )
    .unwrap()
}
