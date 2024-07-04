use std::sync::Arc;

use vulkano::{
    buffer::{BufferContents, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBuffer, CommandBufferBeginInfo,
        CommandBufferInheritanceInfo, CommandBufferLevel, CommandBufferUsage,
        RecordingCommandBuffer,
    },
    device::Queue,
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::Subpass,
};

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Vert {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

pub struct BasicPSO {
    gfx_queue: Arc<Queue>,
    subpass: Subpass,
    pub pipeline: Arc<GraphicsPipeline>,
    cb_allocator: Arc<StandardCommandBufferAllocator>,
}

impl BasicPSO {
    pub fn new(
        gfx_queue: Arc<Queue>,
        subpass: Subpass,
        cb_allocator: Arc<StandardCommandBufferAllocator>,
    ) -> Self {
        let device = gfx_queue.device();
        let vs = vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();

        let vertex_input_state = Vert::per_vertex().definition(&vs).unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();

        let pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState {
                    topology: PrimitiveTopology::TriangleList,
                    ..Default::default()
                }),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState {
                    rasterization_samples: subpass.num_samples().unwrap(),
                    ..Default::default()
                }),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState {
                        blend: Some(AttachmentBlend::alpha()),
                        ..Default::default()
                    },
                )),
                depth_stencil_state: None,
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.clone().into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap();

        Self {
            gfx_queue,
            subpass,
            pipeline,
            cb_allocator,
        }
    }

    /// Builds a secondary command buffer that draws the triangle on the current subpass.
    pub fn draw<V>(
        &self,
        viewport_dimensions: [u32; 2],
        vertices: Subbuffer<[V]>,
    ) -> Arc<CommandBuffer> {
        let mut builder = RecordingCommandBuffer::new(
            self.cb_allocator.clone(),
            self.gfx_queue.queue_family_index(),
            CommandBufferLevel::Secondary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::MultipleSubmit,
                inheritance_info: Some(CommandBufferInheritanceInfo {
                    render_pass: Some(self.subpass.clone().into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();

        builder
            .set_viewport(
                0,
                [Viewport {
                    offset: [0.0, 0.0],
                    extent: [viewport_dimensions[0] as f32, viewport_dimensions[1] as f32],
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap()
            .bind_vertex_buffers(0, vertices.clone())
            .unwrap();

        unsafe {
            builder.draw(vertices.len() as u32, 1, 0, 0).unwrap();
        }

        builder.end().unwrap()
    }
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec3 color;
            layout(location = 0) out vec3 v_color;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                v_color = color;
            }
        ",
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450

            layout(location = 0) in vec3 v_color;
            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(v_color, 1.0);
            }
        ",
    }
}
