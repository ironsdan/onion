use std::sync::Arc;

use vulkano::{
    buffer::BufferContents,
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::ViewportState,
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::Subpass,
    Validated,
};

#[derive(BufferContents, Vertex, Default)]
#[repr(C)]
pub struct Vert {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

pub fn ddd_pso(
    device: Arc<Device>,
    subpass: Subpass,
) -> Result<Arc<GraphicsPipeline>, Validated<vulkano::VulkanError>> {
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
        // Since we only have one pipeline in this example, and thus one pipeline layout,
        // we automatically generate the creation info for it from the resources used in the
        // shaders. In a real application, you would specify this information manually so that
        // you can re-use one layout in multiple pipelines.
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            // How vertex data is read from the vertex buffers into the vertex shader.
            vertex_input_state: Some(vertex_input_state),
            // How vertices are arranged into primitive shapes.
            // The default primitive shape is a triangle.
            input_assembly_state: Some(InputAssemblyState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            }),
            // How primitives are transformed and clipped to fit the framebuffer.
            // We use a resizable viewport, set to draw over the entire window.
            viewport_state: Some(ViewportState::default()),
            // How polygons are culled and converted into a raster of pixels.
            // The default value does not perform any culling.
            rasterization_state: Some(RasterizationState::default()),
            // How multiple fragment shader samples are converted to a single pixel value.
            // The default value does not perform any multisampling.
            multisample_state: Some(MultisampleState {
                rasterization_samples: subpass.num_samples().unwrap(),
                ..Default::default()
            }),
            // How pixel values are combined with the values already present in the framebuffer.
            // The default value overwrites the old value with the new one, without any
            // blending.
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            // Dynamic states allows us to specify parts of the pipeline settings when
            // recording the command buffer, before we perform drawing.
            // Here, we specify that the viewport should be dynamic.
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout (location = 0) in vec3 position;
            layout (location = 1) in vec3 normal;
            layout (location = 2) in vec3 color;

            layout(push_constant) uniform constants {
                vec4 data;
                mat4 render_matrix;
            } pc;

            layout (location = 0) out vec3 out_color;

            void main() {
                vec4 t = pc.render_matrix * vec4(position, 1.0f);
                gl_Position = t;
                out_color = color;
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

            layout(push_constant) uniform constants {
                vec4 data;
                mat4 render_matrix;
            } pc;

            layout(location = 0) out vec4 f_color;

            void main() {
                // vec2 point = pc.data.xy;
                // float d = distance(point, vec2(gl_FragCoord.x/pc.data.z, gl_FragCoord.y/pc.data.w));    
                // f_color = vec4(vec3(d), 1.0);
                f_color = vec4(v_color, 1.0);
            }
        ",
    }
}
