use std::env;
use std::rc::Rc;
use image::{DynamicImage, RgbaImage};
use wgpu::{BindGroup, BindGroupLayout};
use crate::pos::Tile;
use crate::window::{Texture, WindowContext};

pub struct TextureAtlas {
    uvs: Box<[Uv]>,
    uv_indexes: Box<[usize]>,
    tex: Texture,
    pub bind_group: BindGroup,
    pub layout: BindGroupLayout,
}

#[derive(Copy, Clone, Default)]
pub struct Uv {
    x: f32,
    y: f32,
    size: f32
}

struct AtlasBuilder {
    ctx: Rc<WindowContext>,
    texture: Vec<u8>,
    full_width: usize,
    full_height: usize,

    x: usize,
    y: usize,
    row_height: usize,
}

impl AtlasBuilder {
    fn new(ctx: Rc<WindowContext>, width: usize, height: usize) -> Self {
        AtlasBuilder {
            ctx,
            texture: vec![0; 4 * width * height],
            full_width: width,
            full_height: height,
            x: 0,
            y: 0,
            row_height: 0,
        }
    }

    fn load_file(&mut self, name: &str) -> Uv {
        let bytes = load_binary(name, "assets");  // TODO: reuse allocation
        let img = image::load_from_memory(&bytes).expect("Failed to decode image.");
        self.load(&img)
    }

    fn load(&mut self, img: &DynamicImage) -> Uv {
        let rgba = img.to_rgba8();
        let raw = rgba.as_raw();
        let img_width = rgba.width() as usize;

        // TODO: height check.
        if img_width > self.full_width {
            panic!("Image too wide for atlas");
        }

        if self.x + img_width > self.full_width {
            self.x = 0;
            self.y += self.row_height + 1;
            self.row_height = 0;
        }

        self.row_height = self.row_height.max(rgba.height() as usize);

        for y in 0..(rgba.height() as usize) {
            let row_len = img_width * 4;
            let target_start =((self.y + y) * self.full_width + self.x) * 4;
            let target = &mut self.texture.as_mut_slice()[target_start..(target_start + row_len)];
            let source_start = y * row_len;
            let source = &raw[source_start..(source_start + row_len)];
            target.copy_from_slice(source);
        }

        assert_eq!(img.width(), img.height());  // Only squares for now.
        let uv = Uv {
            x: self.x as f32 / self.full_width as f32,
            y: self.y as f32 / self.full_height as f32,
            size: img_width as f32 / self.full_width as f32,
        };

        self.x += img_width + 1;
        uv
    }

    fn debug_save(&self, path: &str) {
        self.as_image().save(path).unwrap();
    }

    fn as_image(&self) -> DynamicImage {
        let img = RgbaImage::from_vec(self.full_width as u32, self.full_height as u32, self.texture.clone()).unwrap();
        DynamicImage::from(img)
    }

    fn bake(self) -> Texture {
        Texture::from_image(&self.ctx.device, &self.ctx.queue, &self.as_image(), Some("atlas"))
    }
}

impl TextureAtlas {
    pub fn new(ctx: Rc<WindowContext>) -> Self {
        let mut atlas = AtlasBuilder::new(ctx.clone(), 16 * 8, 16 * 8);
        let stone_uv = atlas.load_file("stone.png");
        let grass_uv = atlas.load_file("grass.png");
        let dirt_uv = atlas.load_file("dirt.png");
        let mut blocks: Vec<usize> = vec![];
        blocks.extend([0; 6]);
        blocks.extend([1; 6]);
        blocks.extend([2; 6]);
        blocks.extend([3, 2, 2, 2, 2, 2]);
        atlas.debug_save("target/atlas.png");

        let tex = atlas.bake();
        let layout = ctx.bind_group_layout_texture();
        TextureAtlas {
            uv_indexes: blocks.into_boxed_slice(),
            uvs: vec![Default::default(), stone_uv, dirt_uv, grass_uv].into_boxed_slice(),
            bind_group: ctx.bind_group_texture(&layout, &tex),
            tex,
            layout
        }
    }

    pub fn get(&self, block: Tile, face: usize) -> &Uv {
        let index = (block.0 as usize * 6) + face;
        &self.uvs[self.uv_indexes[index]]
    }
}

impl Uv {
    pub fn top_left(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    pub fn top_right(&self) -> [f32; 2] {
        [self.x + self.size, self.y]
    }

    pub fn bottom_left(&self) -> [f32; 2] {
        [self.x, self.y + self.size]
    }

    pub fn bottom_right(&self) -> [f32; 2] {
        [self.x + self.size, self.y + self.size]
    }
}

pub fn load_binary(file_name: &str, dir: &str) -> Vec<u8> {
    let path = std::path::Path::new(dir)
        .join(file_name);
    let data = std::fs::read(path);

    match data {
        Ok(data) => data,
        Err(e) => {
            println!("CWD: {:?}", env::current_dir());
            panic!("Failed load_binary {}/{}: {}", dir, file_name, e);
        }
    }
}
