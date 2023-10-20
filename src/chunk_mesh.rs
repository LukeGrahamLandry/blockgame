use std::collections::HashMap;
use std::rc::Rc;
use glam::{Mat4, Vec3, Vec4};
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

    const CHUNK_SCALE: f32 = 16.0;

    pub fn update_mesh(&mut self, pos: ChunkPos, chunk: &Chunk) {
        self.chunks.insert(pos, self.build_mesh(pos, chunk));
    }

    fn build_mesh(&self, pos: ChunkPos, chunk: &Chunk) -> Mesh {
        let mut mesh = MeshBuilder::default();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let pos = LocalPos::new(x, y, z);
                    let tile = chunk.get(pos);
                    if tile.0 == 1 {
                        mesh.add_cube(pos.normalized() * Self::CHUNK_SCALE);
                    }
                }
            }
        }

        self.init_mesh(&mesh.vert, &mesh.indi, Self::translate(pos))
    }

    fn translate(pos: ChunkPos) -> Mat4 {
        let offset = Vec3::new(pos.x as f32 * Self::CHUNK_SCALE, pos.y as f32 * Self::CHUNK_SCALE, pos.z as f32 * Self::CHUNK_SCALE);
        Mat4::from_translation(offset)
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

#[derive(Default)]
struct MeshBuilder {
    vert: Vec<ModelVertex>,
    indi: Vec<i32>,
}

impl MeshBuilder {
    fn add_cube(&mut self, pos: Vec3) {
        let down_close_left = [0.0, 0.0, 0.0];
        let down_close_right = [0.0, 0.0, 1.0];
        let down_far_left = [1.0, 0.0, 0.0];
        let down_far_right = [1.0, 0.0, 1.0];
        let up_close_left = [0.0, 1.0, 0.0];
        let up_close_right = [0.0, 1.0, 1.0];
        let up_far_left = [1.0, 1.0, 0.0];
        let up_far_right = [1.0, 1.0, 1.0];

        // bottom
        self.add_quad(pos, down_close_left, down_close_right, down_far_left, down_far_right);

        // far
        self.add_quad(pos, up_far_left, up_far_right, down_far_left, down_far_right);

        // close
        self.add_quad(pos, up_close_left, up_close_right, down_close_left, down_close_right);

        // left
        self.add_quad(pos, up_close_left, up_far_left, down_close_left, down_far_left);

        // right
        self.add_quad(pos, up_close_right, up_far_right, down_close_right, down_far_right);

        // top
        self.add_quad(pos, up_close_left, up_close_right, up_far_left, up_far_right);
    }

    // top left, top right, bottom left, bottom right
    fn add_quad(&mut self, pos: Vec3, a: impl Into<Vec3>, b: impl Into<Vec3> + Copy, c: impl Into<Vec3> + Copy, d: impl Into<Vec3>) {
        self.add_triangle(pos, a, b, c);
        self.add_triangle(pos, b, d, c);
    }

    fn add_triangle(&mut self, v: Vec3, a: impl Into<Vec3>, b: impl Into<Vec3>, c: impl Into<Vec3>) {
        self.vert.push(ModelVertex {
            position: Vec4::from(((a.into() + v), 1.0)).to_array(),
        });
        self.vert.push(ModelVertex {
            position: Vec4::from(((b.into() + v), 1.0)).to_array(),
        });
        self.vert.push(ModelVertex {
            position: Vec4::from(((c.into() + v), 1.0)).to_array(),
        });

        self.indi.push(self.indi.len() as i32);
        self.indi.push(self.indi.len() as i32);
        self.indi.push(self.indi.len() as i32);
    }
}