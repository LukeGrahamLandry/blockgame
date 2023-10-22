use std::collections::HashMap;
use crate::atlas::{AtlasBuilder, Uv};
use std::fmt::Write;
use std::fs;

struct BlockSet {
    atlas: AtlasBuilder,
    uv_mod: String,
    tiles_mod: String,
    all_uvs: Vec<String>,
    atlas_data: String,
    renderers: Vec<String>,
    uv_cache: HashMap<String, (Uv, usize)>,
    solid_tile_count: usize,
    custom_tile_count: usize,
    tests: String
}

pub fn gen(out_dir: &str) {
    let mut blocks = BlockSet::new();
    blocks.build();
    fs::write(format!("{}/gen.rs", out_dir), blocks.code()).unwrap();
    blocks.atlas.save(&format!("{}/atlas.png", out_dir));
}

impl BlockSet {
    fn new() -> Self {
        Self {
            atlas: AtlasBuilder::new(16 * 8, 16 * 8),
            uv_mod: String::new(),
            tiles_mod: "".to_string(),
            all_uvs: vec!["Uv { x: 0.0, y: 0.0, size: 0.0 }".to_string()],
            atlas_data: "".to_string(),
            renderers: vec![],
            uv_cache: Default::default(),
            // Index zero reserved for air.
            solid_tile_count: 1,
            custom_tile_count: 1,
            tests: "".to_string(),
        }
    }

    fn build(&mut self) {
        self.cube("stone.png");
        self.cube("dirt.png");
        self.grass("grass", "grass.png", "dirt.png", "dirt.png");
        self.cube("leaf.png");
        self.pillar("log", "log_top.png", "log_side.png");

        self.simple_custom("sapling.png");
        self.simple_custom("wheat.png");
    }

    fn code(&self) -> String {
        format!(r##"
        pub mod uvs {{
            use common::atlas::Uv;
            pub const ALL: [&'static Uv; {}] = [{}];
            {}
        }}

        pub mod tiles {{
            use common::pos::Tile;
            pub const SOLID_COUNT: usize = {};
            pub const CUSTOM_COUNT: usize = {};
            {}
        }}

        pub mod render {{
            use crate::chunk_mesh::renderers::*;
            pub const FUNCS: [CustomRenderFn; {}] = [&air, {}];
        }}

        use common::atlas::*;
        pub fn get_atlas_data() -> AtlasData {{
            let mut indexes: Vec<usize> = vec![];
            indexes.extend([0; 6]);  // Placeholder
            {}
            AtlasData {{
                uv_indexes: indexes.into_boxed_slice()
            }}
        }}

        #[test]
        fn generated_test() {{
        {}
        }}

        "##, self.all_uvs.len(), self.all_uvs.iter().map(|s| format!("&{},", s)).collect::<String>(), self.uv_mod,
                self.solid_tile_count - 1, self.custom_tile_count - 1, self.tiles_mod,
                self.renderers.len() + 1, self.renderers.iter().map(|s| format!("&{},", s)).collect::<String>(),
                self.atlas_data,
                self.tests
        )
    }

    fn cube(&mut self, side: &str) {
        let uv = self.load_uv(side);
        writeln!(self.atlas_data, "indexes.extend([{}; 6]);  // cube: {}",uv.1, side).unwrap();
        self.tile(&side[0..side.len()-4], self.solid_tile_count, true);
        self.solid_tile_count += 1;
    }

    fn grass(&mut self, name: &str, top: &str, side: &str, bottom: &str) {
        let top= self.load_uv(top);
        let side = self.load_uv(side);
        let bottom= self.load_uv(bottom);
        writeln!(self.atlas_data, "indexes.extend([{0}, {1}, {2}, {2}, {2}, {2}]);  // grass: {3}", top.1, bottom.1, side.1, name).unwrap();
        self.tile(name, self.solid_tile_count, true);
        self.solid_tile_count += 1;
    }

    fn pillar(&mut self, name: &str, top: &str, side: &str) {
        let top= self.load_uv(top);
        let side = self.load_uv(side);
        writeln!(self.atlas_data, "indexes.extend([{0}, {0}, {1}, {1}, {1}, {1}]);  // pillar: {2}",top.1, side.1, name).unwrap();
        self.tile(name, self.solid_tile_count, true);
        self.solid_tile_count += 1;
    }

    // This is a little weird cause I don't use the Uv until later. It's just convenient to write it here.
    fn simple_custom(&mut self, name: &str) {
        let uv = self.load_uv(name);
        let name = &name[0..name.len()-4];
        self.tile(name, self.custom_tile_count, false);
        self.custom_tile_count += 1;
        self.renderers.push(name.to_string());
        writeln!(self.tests, "assert!(fn_eq(render::FUNCS[tiles::{0}.index()], &crate::chunk_mesh::renderers::{0}));"
                 , name).unwrap();


        writeln!(self.atlas_data, "indexes.extend([{}; 6]); // temp custom solid {}", uv.1, name).unwrap();
        self.tile(&format!("{}_solid", name), self.solid_tile_count, true);
        self.solid_tile_count += 1;
    }

    fn tile(&mut self, name: &str, index: usize, solid: bool) {
        writeln!(self.tiles_mod, "pub const {}: Tile = Tile::new({}, {});", name, index, solid).unwrap();
    }

    fn load_uv(&mut self, path: &str) -> (Uv, usize) {
        assert!(path.ends_with(".png"));
        match self.uv_cache.get(path) {
            None => {
                println!("cargo:rerun-if-changed=assets/{}", path);
                let uv = self.atlas.load_file(path);
                let index = self.uv_cache.len() + 1;
                let name = &path[0..path.len()-4];
                self.all_uvs.push(name.to_string());
                self.uv_cache.insert(path.to_string(), (uv, index));
                writeln!(self.uv_mod,
                         "pub const {}: Uv = Uv {{ x: {}f32, y: {}f32, size: {}f32 }};",
                         name, uv.x, uv.y, uv.size
                ).unwrap();
                (uv, index)
            }
            Some(uv) => *uv,
        }
    }
}
