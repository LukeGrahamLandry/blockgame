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
        // TODO: easy culling based on ChunkPos and camera direction.
        for mesh in self.chunks.values() {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(1, &mesh.info_bind_group, &[]);
            render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
        }
    }

    const CHUNK_SCALE: f32 = CHUNK_SIZE as f32;

    pub fn update_mesh(&mut self, pos: ChunkPos, chunk: &Chunk) {
        self.chunks.insert(pos, self.build_mesh(pos, chunk));
    }

    fn build_mesh(&self, pos: ChunkPos, chunk: &Chunk) -> Mesh {
        let mut mesh = MeshBuilder::default();  // TODO: reuse allocation

        // TODO: could use wrapping and stay unsigned since negative becomes really high positive
        // TODO: you already know in the loop which are the edge so maybe treat those differently and the don't need the branching here.
        let empty = |x: isize, y: isize, z: isize| {
            let is = CHUNK_SIZE as isize;
            if x >= is || y >= is || z >= is || x < 0 || y < 0 || z < 0 {
                return true;
            }
            let pos = LocalPos::new(x as usize, y as usize, z as usize);
            let tile = chunk.get(pos);
            tile.0 == 0
        };

        let mut count = 0;
        for x in 0..(CHUNK_SIZE as isize) {
            for y in 0..(CHUNK_SIZE as isize) {
                for z in 0..(CHUNK_SIZE as isize) {
                    if !empty(x, y, z) {
                        let pos = LocalPos::new(x as usize, y as usize, z as usize);
                        let top = empty(x, y + 1, z);
                        let right = empty(x, y, z + 1);
                        let far = empty(x + 1, y, z);
                        let bottom = empty(x, y - 1, z);
                        let left = empty(x, y, z - 1);
                        let close = empty(x - 1, y, z);
                        mesh.add_cube(pos.normalized() * Self::CHUNK_SCALE, top, bottom, left, right, close, far);
                        count += 1;
                    }
                }
            }
        }

        println!("Mesh({}, {}, {}): {} vertices, {} indices, {} cubes.", pos.x, pos.y, pos.z, mesh.vert.len(), mesh.indi.len(), count);

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
            num_elements: indi.len() as u32,
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
    fn add_cube(&mut self, pos: Vec3, top: bool, bottom: bool, left: bool, right: bool, close: bool, far: bool) {
        let mut down_close_left = 0;
        let mut down_close_right = 0;
        let mut down_far_left = 0;
        let mut down_far_right = 0;
        let mut up_close_left = 0;
        let mut up_close_right = 0;
        let mut up_far_left = 0;
        let mut up_far_right = 0;

        if bottom || close || left {
            down_close_left = self.vertex(pos, [0.0, 0.0, 0.0]);
        }
        if bottom || close || right {
            down_close_right = self.vertex(pos, [0.0, 0.0, 1.0]);
        }
        if bottom || far || left {
            down_far_left = self.vertex(pos, [1.0, 0.0, 0.0]);
        }
        if bottom || far || right {
            down_far_right = self.vertex(pos, [1.0, 0.0, 1.0]);
        }
        if top || close || left {
            up_close_left = self.vertex(pos, [0.0, 1.0, 0.0]);
        }
        if top || close || right {
            up_close_right = self.vertex(pos, [0.0, 1.0, 1.0]);
        }
        if top || far || left {
            up_far_left = self.vertex(pos, [1.0, 1.0, 0.0]);
        }
        if top || far || right {
            up_far_right = self.vertex(pos, [1.0, 1.0, 1.0]);
        }

        if bottom {
            self.add_quad(pos, down_close_left, down_close_right, down_far_left, down_far_right);
        }

        if far {
            self.add_quad(pos, up_far_left, up_far_right, down_far_left, down_far_right);
        }

        if close {
            self.add_quad(pos, up_close_left, up_close_right, down_close_left, down_close_right);
        }

        if left {
            self.add_quad(pos, up_close_left, up_far_left, down_close_left, down_far_left);
        }

        if right {
            self.add_quad(pos, up_close_right, up_far_right, down_close_right, down_far_right);
        }

        if top {
            self.add_quad(pos, up_close_left, up_close_right, up_far_left, up_far_right);
        }
    }

    fn vertex(&mut self, pos: Vec3, a: impl Into<Vec3>) -> i32 {
        self.vert.push(ModelVertex {
            position: Vec4::from(((a.into() + pos), 1.0)).to_array(),
        });
        (self.vert.len() - 1) as i32
    }

    // top left, top right, bottom left, bottom right
    fn add_quad(&mut self, pos: Vec3, a: i32, b: i32, c: i32, d: i32) {
        self.add_triangle(pos, a, b, c);
        self.add_triangle(pos, b, d, c);
    }

    fn add_triangle(&mut self, v: Vec3, a: i32, b: i32, c: i32) {
        self.indi.push(a);
        self.indi.push(b);
        self.indi.push(c);
    }
}
