use std::collections::HashMap;
use std::rc::Rc;
use glam::Mat4;
use wgpu::{BindGroupLayout, RenderPass};
use crate::pos::{ChunkPos, Chunk, CHUNK_SIZE, LocalPos};
use crate::window::{Mesh, MeshUniform, ModelVertex, ref_to_bytes, slice_to_bytes, WindowContext};

pub struct ChunkList {
    chunks: HashMap<ChunkPos, Mesh>,
    layout: BindGroupLayout,
    ctx: Rc<WindowContext>
}

impl ChunkList {
    pub fn new(ctx: Rc<WindowContext>, layout: BindGroupLayout) -> Self {
        ChunkList {
            chunks: Default::default(),
            layout,
            ctx,
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        for mesh in self.chunks.values() {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(1, &mesh.info_bind_group, &[]);
            render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
    }

    pub fn update_mesh(&mut self, pos: ChunkPos, chunk: &Chunk) {
        self.chunks.insert(pos, self.build_mesh(chunk));
    }

    fn build_mesh(&self, chunk: &Chunk) -> Mesh {
        let mut vert = vec![];

        let mut indi = vec![];

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let pos = LocalPos::new(x, y, z);
                    let tile = chunk.get(pos);
                    if tile.0 == 1 {
                        let xv = x as f32 / CHUNK_SIZE as f32 * 10.0;
                        let yv = y as f32 / CHUNK_SIZE as f32 * 10.0;
                        let zv = z as f32 / CHUNK_SIZE as f32 * 10.0;

                        vert.push(ModelVertex {
                            position: [0.0 + xv, 0.5 + yv, 0.0 + zv, 1.0],
                        });
                        vert.push(ModelVertex {
                            position: [-0.5 + xv, -0.5 + yv, 0.0 + zv, 1.0],
                        });
                        vert.push(ModelVertex {
                            position: [0.5 + xv, -0.5 + yv, 0.0 + zv, 1.0],
                        });

                        indi.push(indi.len() as i32);
                        indi.push(indi.len() as i32);
                        indi.push(indi.len() as i32);
                    }
                }
            }
        }

        self.init_mesh(&vert, &indi, Mat4::IDENTITY)
    }

    fn init_mesh(&self, vert: &[ModelVertex], indi: &[i32], transform: Mat4) -> Mesh {
        let vertex_buffer = self.ctx.buffer_init(
            "tri", slice_to_bytes(vert), wgpu::BufferUsages::VERTEX
        );
        let index_buffer = self.ctx.buffer_init(
            "tri", slice_to_bytes(indi), wgpu::BufferUsages::INDEX
        );

        let transform = MeshUniform {
            transform: transform.to_cols_array_2d(),
        };

        let info_buffer = self.ctx.buffer_init(
            "mesh_info", ref_to_bytes(&transform),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        );

        let info_bind_group = self.ctx.bind_group("mesh_info", &self.layout, &[
            info_buffer.as_entire_binding()
        ]);

        Mesh {
            name: "".to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: vert.len() as u32,
            transform,
            info_buffer,
            info_bind_group,
        }
    }
}
