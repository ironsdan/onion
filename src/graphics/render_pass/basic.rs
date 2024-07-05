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
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount},
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
    Validated, ValidationError, VulkanError,
};

pub struct RenderPassBasic {
    pub gfx_queue: Arc<Queue>,
    pub render_pass: Arc<RenderPass>,
    pub cb_allocator: Arc<dyn CommandBufferAllocator>,
}

impl RenderPassBasic {
    pub fn new(gfx_queue: Arc<Queue>, format: Format) -> Result<Self, Validated<VulkanError>> {
        let device = gfx_queue.device().clone();
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
        clear_color: [f32; 4],
        before_future: F,
        final_image: Arc<Image>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<BasicFrame, Validated<VulkanError>>
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
        Ok(BasicFrame {
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

pub struct BasicFrame<'a> {
    system: &'a mut RenderPassBasic,
    num_pass: u8,
    framebuffer: Arc<Framebuffer>,
    before_main_cb_future: Option<Box<dyn GpuFuture>>,
    command_buffer: Option<RecordingCommandBuffer>,
}

impl<'a> BasicFrame<'a> {
    pub fn next_pass<'f>(&'f mut self) -> Result<Option<BasicPass<'f, 'a>>, Box<ValidationError>> {
        Ok(
            match {
                let current_pass = self.num_pass;
                self.num_pass += 1;
                current_pass
            } {
                0 => Some(BasicPass::Draw(BasicDrawPass { frame: self })),
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
                    Some(BasicPass::Finished(after_main_cb.boxed()))
                }
                _ => None,
            },
        )
    }
}

/// Struct provided to the user that allows them to customize or handle the pass.
pub enum BasicPass<'f, 's: 'f> {
    Draw(BasicDrawPass<'f, 's>),
    Finished(Box<dyn GpuFuture>),
}

/// Allows the user to draw objects on the scene.
pub struct BasicDrawPass<'f, 's: 'f> {
    frame: &'f mut BasicFrame<'s>,
}

impl<'f, 's: 'f> BasicDrawPass<'f, 's> {
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

pub struct RenderPassBasicMSAA {
    pub gfx_queue: Arc<Queue>,
    pub render_pass: Arc<RenderPass>,
    pub cb_allocator: Arc<dyn CommandBufferAllocator>,
}

impl RenderPassBasicMSAA {
    pub fn new(gfx_queue: Arc<Queue>, format: Format) -> Result<Self, Validated<VulkanError>> {
        let device = gfx_queue.device().clone();
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
        clear_color: [f32; 4],
        before_future: F,
        final_image: Arc<Image>,
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<BasicMSAAFrame, Validated<VulkanError>>
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
        Ok(BasicMSAAFrame {
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

pub struct BasicMSAAFrame<'a> {
    system: &'a mut RenderPassBasicMSAA,
    num_pass: u8,
    framebuffer: Arc<Framebuffer>,
    before_main_cb_future: Option<Box<dyn GpuFuture>>,
    command_buffer: Option<RecordingCommandBuffer>,
}

impl<'a> BasicMSAAFrame<'a> {
    pub fn next_pass<'f>(
        &'f mut self,
    ) -> Result<Option<BasicMSAAPass<'f, 'a>>, Box<ValidationError>> {
        Ok(
            match {
                let current_pass = self.num_pass;
                self.num_pass += 1;
                current_pass
            } {
                0 => Some(BasicMSAAPass::Draw(BasicMSAADrawPass { frame: self })),
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
                    Some(BasicMSAAPass::Finished(after_main_cb.boxed()))
                }
                _ => None,
            },
        )
    }
}

/// Struct provided to the user that allows them to customize or handle the pass.
pub enum BasicMSAAPass<'f, 's: 'f> {
    Draw(BasicMSAADrawPass<'f, 's>),
    Finished(Box<dyn GpuFuture>),
}

/// Allows the user to draw objects on the scene.
pub struct BasicMSAADrawPass<'f, 's: 'f> {
    frame: &'f mut BasicMSAAFrame<'s>,
}

impl<'f, 's: 'f> BasicMSAADrawPass<'f, 's> {
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

// TODO Need some way to abstract the frames with similar structure
// to pass the system parameter of Frame, maybe something like:
// pub trait FrameSystem {
//     fn cb_allocator(&self) -> Arc<dyn CommandBufferAllocator>;
//     fn queue(&self) -> Arc<Queue>;
//     fn frame<F>(
//         &mut self,
//         clear_color: [f32; 4],
//         before_future: F,
//         final_image: Arc<Image>,
//         memory_allocator: Arc<StandardMemoryAllocator>,
//     ) -> Result<Frame, Validated<VulkanError>>
//     where
//         F: GpuFuture + 'static;
//     fn draw_pass(&self) -> Subpass;
// }
// This doesn't work because FrameSystem can't be made into an object.
// Or at the frame level with:
// pub trait Frame<'a> {
//     fn next_pass<'f>(
//         &'f mut self,
//     ) -> Result<Option<BasicMSAAPass<'f, 'a>>, Box<ValidationError>>;
// }
// Which actually does work but I don't like as much.
