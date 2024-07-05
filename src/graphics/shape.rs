use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::CommandBuffer,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
};

use crate::graphics::pipelines::basic::{BasicPSO, Vert};
use crate::graphics::Color;

pub struct Square {
    size: f32,
    color: Color,
}

impl Square {
    pub fn new(size: f32, color: Color) -> Self {
        Square { size, color }
    }

    pub fn draw(
        &self,
        memory_allocator: Arc<dyn MemoryAllocator>,
        pipeline: &mut BasicPSO,
        viewport: [u32; 2],
    ) -> Arc<CommandBuffer> {
        let vertices = [
            Vert {
                position: [-self.size, -self.size],
                color: self.color.into(),
            },
            Vert {
                position: [self.size, self.size],
                color: self.color.into(),
            },
            Vert {
                position: [-self.size, self.size],
                color: self.color.into(),
            },
            Vert {
                position: [-self.size, -self.size],
                color: self.color.into(),
            },
            Vert {
                position: [self.size, -self.size],
                color: self.color.into(),
            },
            Vert {
                position: [self.size, self.size],
                color: self.color.into(),
            },
        ];

        let vb = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .unwrap();

        pipeline.draw(viewport, vb)
    }
}
