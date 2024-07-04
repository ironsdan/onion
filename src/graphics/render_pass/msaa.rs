use std::sync::Arc;

use vulkano::{
    command_buffer::{
        allocator::{CommandBufferAllocator, StandardCommandBufferAllocator},
        CommandBuffer, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsage,
        RecordingCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        SubpassEndInfo,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount},
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    pipeline::GraphicsPipeline,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
    Validated, ValidationError, VulkanError,
};

use crate::graphics::pipelines::{self, line::LinePSO};

pub struct RenderPassMSAABasic {
    pub gfx_queue: Arc<Queue>,
    pub render_pass: Arc<RenderPass>,
    pub cb_allocator: Arc<dyn CommandBufferAllocator>,
    // PSOs
    pub line_pso: LinePSO,
    pub texture_pso: Arc<GraphicsPipeline>,
}

impl RenderPassMSAABasic {
    pub fn new(
        device: Arc<Device>,
        gfx_queue: Arc<Queue>,
        format: Format,
    ) -> Result<Self, Validated<VulkanError>> {
        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                intermediary: {
                    format: format,
                    // This has to match the image definition.
                    samples: 4,
                    load_op: Clear,
                    store_op: DontCare,
                },
                color: {
                    format: format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [intermediary],
                color_resolve: [color],
                depth_stencil: {},
            },
        )?;

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        let cb_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let line_pso = pipelines::line::LinePSO::new(device.clone(), subpass.clone());
        let texture_pso = pipelines::texture::texture_pso(device.clone(), subpass.clone())?;

        Ok(Self {
            gfx_queue,
            render_pass,
            line_pso,
            texture_pso,
            cb_allocator,
        })
    }

    pub fn frame<F>(
        &mut self,
        clear_color: [f32; 4],
        before_future: F,
        final_image: Arc<Image>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Frame, Validated<VulkanError>>
    where
        F: GpuFuture + 'static,
    {
        let framebuffer = framebuffer_setup(
            final_image.clone(),
            self.render_pass.clone(),
            memory_allocator.clone(),
        );

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
                clear_values: vec![Some(clear_color.into()), Some(clear_color.into())],

                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            },
            SubpassBeginInfo {
                contents: SubpassContents::SecondaryCommandBuffers,
                ..Default::default()
            },
        )?;
        Ok(Frame {
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

pub struct Frame<'a> {
    system: &'a mut RenderPassMSAABasic,
    num_pass: u8,
    framebuffer: Arc<Framebuffer>,
    before_main_cb_future: Option<Box<dyn GpuFuture>>,
    command_buffer: Option<RecordingCommandBuffer>,
}

impl<'a> Frame<'a> {
    pub fn next_pass<'f>(&'f mut self) -> Result<Option<Pass<'f, 'a>>, Box<ValidationError>> {
        Ok(
            match {
                let current_pass = self.num_pass;
                self.num_pass += 1;
                current_pass
            } {
                0 => Some(Pass::Basic(DrawPass { frame: self })),
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
                    Some(Pass::Finished(after_main_cb.boxed()))
                }
                _ => None,
            },
        )
    }
}

/// Struct provided to the user that allows them to customize or handle the pass.
pub enum Pass<'f, 's: 'f> {
    Basic(DrawPass<'f, 's>),
    Finished(Box<dyn GpuFuture>),
}

/// Allows the user to draw objects on the scene.
pub struct DrawPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> DrawPass<'f, 's> {
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

fn framebuffer_setup(
    image: Arc<Image>,
    render_pass: Arc<RenderPass>,
    memory_allocator: Arc<StandardMemoryAllocator>,
) -> Arc<Framebuffer> {
    let extent = image.extent();
    let intermediary = ImageView::new_default(
        Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: image.format(),
                extent: [extent[0], extent[1], 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                samples: SampleCount::Sample4,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    let view = ImageView::new_default(image.clone()).unwrap();
    Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![intermediary.clone(), view],
            ..Default::default()
        },
    )
    .unwrap()
}
