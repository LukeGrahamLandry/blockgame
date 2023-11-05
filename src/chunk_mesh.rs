use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use glam::{Mat4, Vec3, Vec4};
use wgpu::{BindGroup, BindGroupLayout, RenderPass};
use crate::pos::{ChunkPos, Chunk, CHUNK_SIZE, LocalPos, Direction};
use common::atlas::Uv;
use common::pos::Tile;
use crate::gen;
use crate::window::{Mesh, MeshUniform, ModelVertex, ref_to_bytes, slice_to_bytes, Texture, WindowContext};

pub struct ChunkList {
    chunks: HashMap<ChunkPos, Mesh>,
    layout: BindGroupLayout,
    ctx: Rc<WindowContext>,
    pub builder: MeshBuilder,

    // Old meshes not currently in use. When loading a new chunk, check if there's one here,
    // since I assume it's cheaper to update a buffer than create a new one.
    pub mesh_pool: Vec<Mesh>,
    #[cfg(feature = "profiling")]
    init_count: Cell<usize>,
    #[cfg(feature = "profiling")]
    memory: Cell<u64>
}

impl ChunkList {
    pub fn new(ctx: Rc<WindowContext>, atlas: Rc<TextureAtlas>, layout: BindGroupLayout) -> Self {
        ChunkList {
            chunks: Default::default(),
            layout,
            ctx,
            builder: MeshBuilder {
                atlas,
                vert: Vec::with_capacity(10000),
                indi: Vec::with_capacity(10000),
            },
            mesh_pool: Vec::with_capacity(Self::MAX_ALLOC_POOL),
            #[cfg(feature = "profiling")]
            init_count: Cell::new(0),
            #[cfg(feature = "profiling")]
            memory: Cell::new(0),
        }
    }

    pub fn remove(&mut self, pos: ChunkPos) {
        let old = self.chunks.remove(&pos);
        self.recycle(old);
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, player: ChunkPos) {
        // TODO: easy culling based on ChunkPos and camera direction.
        for (pos, mesh) in self.chunks.iter() {
            if player.axis_distance(pos) > 5 {
                continue;
            }
            mesh.render(render_pass);
        }
    }

    const CHUNK_SCALE: f32 = CHUNK_SIZE as f32;
    const MAX_ALLOC_POOL: usize = 1000;  // TODO: need to set this dynamically based on render distance.

    pub fn update_mesh(&mut self, pos: ChunkPos, chunk: &Chunk) {
        // println!("Meshes: {} + {}", self.chunks.len(), self.mesh_pool.len());
        let old = if let Some(mesh) = self.build_mesh(pos, chunk) {
            self.chunks.insert(pos, mesh)
        } else {
            self.chunks.remove(&pos)
        };
        self.recycle(old);
    }

    pub fn recycle(&mut self, mesh: Option<Mesh>) {
        if let Some(old) = mesh {
            if self.mesh_pool.len() < Self::MAX_ALLOC_POOL {
                self.mesh_pool.push(old);
            } else {
                print!("full mesh pool");
            }
        }
    }

    fn build_mesh(&mut self, pos: ChunkPos, chunk: &Chunk) -> Option<Mesh> {
        self.builder.clear();

        // TODO: could use wrapping and stay unsigned since negative becomes really high positive
        // TODO: you already know in the loop which are the edge so maybe treat those differently and the don't need the branching here.
        let empty = |x: isize, y: isize, z: isize| {
            let is = CHUNK_SIZE as isize;
            if x >= is || y >= is || z >= is || x < 0 || y < 0 || z < 0 {
                return true;
            }
            let pos = LocalPos::new(x as usize, y as usize, z as usize);
            let tile = chunk.get(pos);
            !tile.solid()
        };

        let mut count = 0;
        for x in (0..(CHUNK_SIZE as isize))  {
            for y in 0..(CHUNK_SIZE as isize) {
                for z in 0..(CHUNK_SIZE as isize)  {
                    let pos = LocalPos::new(x as usize, y as usize, z as usize);
                    let tile = chunk.get(pos);
                    if tile.solid() {
                        debug_assert!(tile.index() <= gen::tiles::SOLID_COUNT, "Invalid tile {:?} at {:?} {:?}. Damn you lua!", tile, chunk.pos, pos);
                        // TODO: use DirectionSet?
                        let top = empty(x, y + 1, z);
                        let right = empty(x, y, z + 1);
                        let far = empty(x + 1, y, z);
                        let bottom = empty(x, y - 1, z);
                        let left = empty(x, y, z - 1);
                        let close = empty(x - 1, y, z);
                        self.builder.add_cube(tile, pos.normalized() * Self::CHUNK_SCALE, top, bottom, left, right, close, far);
                        count += 1;
                    } else if tile.custom_render() {
                        debug_assert!(tile.index() <= gen::tiles::CUSTOM_COUNT, "Invalid tile {:?} at {:?} {:?}. Damn you lua!", tile, chunk.pos, pos);
                        let func = gen::render::FUNCS[tile.index()];
                        func(&mut self.builder, pos.normalized() * Self::CHUNK_SCALE);
                    }
                }
            }
        }

        if self.builder.indi.is_empty() {
            return None;
        }

        // println!("Mesh({}, {}, {}): {} vertices, {} indices, {} cubes.", pos.x, pos.y, pos.z, self.builder.vert.len(), self.builder.indi.len(), count);

        Some(match self.mesh_pool.pop() {
            None => self.init_mesh(&self.builder.vert, &self.builder.indi, Self::translate(pos)),
            Some(mut mesh) => {
                self.reuse_mesh(&mut mesh, &self.builder.vert, &self.builder.indi, Self::translate(pos));
                mesh
            }
        })
    }

    fn translate(pos: ChunkPos) -> Mat4 {
        let offset = Vec3::new(pos.x as f32 * Self::CHUNK_SCALE, pos.y as f32 * Self::CHUNK_SCALE, pos.z as f32 * Self::CHUNK_SCALE);
        Mat4::from_translation(offset)
    }

    // This checks that there's enough space in the buffer before writing. Resizing will amortize.
    pub(crate) fn reuse_mesh(&self, mesh: &mut Mesh, vert: &[ModelVertex], indi: &[u32], transform: Mat4)  {
        let vert_b = slice_to_bytes(vert);
        let indi_b = slice_to_bytes(indi);

        if vert_b.len() <= mesh.vertex_buffer.size() as usize {
            self.ctx.write_buffer(&mesh.vertex_buffer, vert_b);
        } else {
            #[cfg(feature = "profiling")]
            self.memory.set(self.memory.get() - mesh.vertex_buffer.size());
            mesh.vertex_buffer = self.ctx.buffer_init(
                "tri", vert_b, wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
            );
            #[cfg(feature = "profiling")]
            self.memory.set(self.memory.get() + mesh.vertex_buffer.size());
        }

        if indi_b.len() <= mesh.index_buffer.size() as usize {
            self.ctx.write_buffer(&mesh.index_buffer, indi_b);
        } else {
            #[cfg(feature = "profiling")]
            self.memory.set(self.memory.get() - mesh.index_buffer.size());
            mesh.index_buffer = self.ctx.buffer_init(
                "tri", indi_b, wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
            );
            #[cfg(feature = "profiling")]
            self.memory.set(self.memory.get() + mesh.index_buffer.size());
        }

        let transform = MeshUniform {
            transform: transform.to_cols_array_2d(),
        };

        // Never need to resize, info length is fixed.
        self.ctx.write_buffer(&mesh.info_buffer, ref_to_bytes(&transform));

        mesh.num_elements = indi.len() as u32;
    }

    // TODO: move this method since entity wants it too
    pub fn init_mesh(&self, vert: &[ModelVertex], indi: &[u32], transform: Mat4) -> Mesh {
        #[cfg(feature = "profiling")]
        self.init_count.set(self.init_count.get() + 1);

        let vertex_buffer = self.ctx.buffer_init(
            "tri", slice_to_bytes(vert), wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
        );
        let index_buffer = self.ctx.buffer_init(
            "tri", slice_to_bytes(indi), wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST
        );

        let transform = MeshUniform {
            transform: transform.to_cols_array_2d(),
        };

        let info_buffer = self.ctx.buffer_init(
            "mesh_info", ref_to_bytes(&transform),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        );

        #[cfg(feature = "profiling")]
        self.memory.set(self.memory.get() + index_buffer.size() + vertex_buffer.size() + info_buffer.size());

        let info_bind_group = self.ctx.bind_group("mesh_info", &self.layout, &[
            info_buffer.as_entire_binding()
        ]);

        Mesh {
            vertex_buffer,
            index_buffer,
            num_elements: indi.len() as u32,
            transform,
            info_buffer,
            info_bind_group,
        }
    }

    #[cfg(feature = "profiling")]
    pub fn log_profile(&self) {
        // Note: currently MB includes entities since they use the same chunk pool
        //       planning to just use one mesh for all entities and move resizing to mesh struct.
        println!("ChunkRender:\n  - loaded: {}\n  - pool: {}\n  - gpu MB: {}", self.chunks.len(), self.mesh_pool.len(), self.memory.get() / 1024 / 1024);
    }
}

pub struct MeshBuilder {
    pub atlas: Rc<TextureAtlas>,
    pub vert: Vec<ModelVertex>,
    pub indi: Vec<u32>,
}

impl MeshBuilder {
    pub fn clear(&mut self) {
        self.vert.clear();
        self.indi.clear();
    }

    pub fn add_cube(&mut self, tile: Tile, pos: Vec3, top: bool, bottom: bool, left: bool, right: bool, close: bool, far: bool) {
        debug_assert!(tile.solid());
        let down_close_left = [0.0, 0.0, 0.0];
        let down_close_right = [0.0, 0.0, 1.0];
        let down_far_left = [1.0, 0.0, 0.0];
        let down_far_right = [1.0, 0.0, 1.0];
        let up_close_left = [0.0, 1.0, 0.0];
        let up_close_right = [0.0, 1.0, 1.0];
        let up_far_left = [1.0, 1.0, 0.0];
        let up_far_right = [1.0, 1.0, 1.0];

        if bottom {
            let uv = *self.atlas.get(tile, Direction::Down);
            self.add_quad(&uv, pos, down_close_left, down_close_right, down_far_left, down_far_right);
        }

        if far {
            let uv = *self.atlas.get(tile, Direction::North);
            self.add_quad(&uv, pos, up_far_left, up_far_right, down_far_left, down_far_right);
        }

        if close {
            let uv = *self.atlas.get(tile, Direction::South);
            self.add_quad(&uv, pos, up_close_left, up_close_right, down_close_left, down_close_right);
        }

        if left {
            let uv = *self.atlas.get(tile, Direction::West);
            self.add_quad(&uv, pos, up_close_left, up_far_left, down_close_left, down_far_left);
        }

        if right {
            let uv = *self.atlas.get(tile, Direction::East);
            self.add_quad(&uv, pos, up_close_right, up_far_right, down_close_right, down_far_right);
        }

        if top {
            let uv = *self.atlas.get(tile, Direction::Up);
            self.add_quad(&uv, pos, up_close_left, up_close_right, up_far_left, up_far_right);
        }
    }

    fn vertex(&mut self, uv: [f32; 2], pos: Vec3, a: impl Into<Vec3>) -> u32 {
        self.vert.push(ModelVertex {
            position: Vec4::from(((a.into() + pos), 1.0)).to_array(),
            uv,
        });
        (self.vert.len() - 1) as u32
    }

    // top left, top right, bottom left, bottom right
    fn add_quad(&mut self, uv: &Uv, pos: Vec3, a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) {
        let a = self.vertex(uv.top_left(), pos, a);
        let b = self.vertex(uv.top_right(), pos, b);
        let c = self.vertex(uv.bottom_left(), pos, c);
        let d = self.vertex(uv.bottom_right(), pos, d);
        self.add_triangle(a, b, c);
        self.add_triangle(b, d, c);
    }

    fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indi.push(a);
        self.indi.push(b);
        self.indi.push(c);
    }
}

pub struct TextureAtlas {
    _tex: Texture,  // Never need to use this, but it needs to stay alive and not call drop.
    pub bind_group: BindGroup,
    pub layout: BindGroupLayout,
}

impl TextureAtlas {
    pub fn new(ctx: &WindowContext) -> Self {
        let tex = Self::bake(ctx);
        let layout = ctx.bind_group_layout_texture();
        TextureAtlas {
            bind_group: ctx.bind_group_texture(&layout, &tex),
            _tex: tex,
            layout
        }
    }

    fn bake(ctx: &WindowContext) -> Texture {
        let img = image::load_from_memory(gen::ATLAS_PNG).unwrap();
        Texture::from_image(&ctx.device, &ctx.queue, &img, Some("atlas"))
    }

    pub fn get(&self, block: Tile, face: Direction) -> &Uv {
        debug_assert!(block.solid());
        let index = (block.index() * 6) + face as usize;
        &gen::uvs::ALL[gen::uvs::SOLID_INDEXES[index] as usize]
    }
}

pub mod renderers {
    use glam::Vec3;
    use common::atlas::Uv;
    use crate::chunk_mesh::{MeshBuilder};
    use crate::gen::uvs;

    pub type CustomRenderFn = &'static dyn Fn(&mut MeshBuilder, Vec3);

    pub fn air(_: &mut MeshBuilder, _: Vec3) {
        unreachable!()
    }

    pub fn sapling(mesh: &mut MeshBuilder, pos: Vec3) {
        let uv = uvs::sapling;
        // These have x/z swapped so it makes a little cross.
        mesh.add_quad(uv, pos, [0.0, 1.0, 0.5], [1.0, 1.0, 0.5], [0.0, 0.0, 0.5], [1.0, 0.0, 0.5]);
        mesh.add_quad(uv, pos, [0.5, 1.0, 0.0], [0.5, 1.0, 1.0], [0.5, 0.0, 0.0], [0.5, 0.0, 1.0]);
    }

    // TODO: no copy paste! render funcs need to get their tile?
    pub fn wheat(mesh: &mut MeshBuilder, pos: Vec3) {
        plant(mesh, pos,uvs::wheat);
    }
    pub fn wheat1(mesh: &mut MeshBuilder, pos: Vec3) {
        plant(mesh, pos,uvs::wheat1);
    }
    pub fn wheat2(mesh: &mut MeshBuilder, pos: Vec3) {
        plant(mesh, pos,uvs::wheat2);
    }
    pub fn wheat3(mesh: &mut MeshBuilder, pos: Vec3) {
        plant(mesh, pos,uvs::wheat3);
    }

    fn plant(mesh: &mut MeshBuilder, pos: Vec3, uv: &Uv) {
        // This time two quads going across.
        let a = [0.2, 0.8];
        for a in a {
            mesh.add_quad(uv, pos, [0.0, 1.0, a], [1.0, 1.0, a], [0.0, 0.0, a], [1.0, 0.0, a]);
        }
        // Then swap x/z so its like a tick-tac-toe board.
        for a in a {
            mesh.add_quad(uv, pos, [a, 1.0, 0.0], [a, 1.0, 1.0], [a, 0.0, 0.0], [a, 0.0, 1.0]);
        }
    }
}
