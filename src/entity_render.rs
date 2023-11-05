use std::collections::HashMap;
use std::rc::Rc;
use wgpu::RenderPass;
use crate::chunk_mesh::ChunkList;
use crate::window::{Mesh, WindowContext};

// TODO: this needs to be a trait
pub enum EntityInfo {
    None,
    SingleMesh(Mesh)
}

pub struct EntityRender {
    entities: HashMap<i32, EntityInfo>,
    ctx: Rc<WindowContext>,
}

impl EntityRender {
    pub fn new(ctx: Rc<WindowContext>) -> Self {
        Self {
            entities: Default::default(),
            ctx,
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        for info in self.entities.values() {
            match info {
                EntityInfo::None => {}
                EntityInfo::SingleMesh(mesh) => mesh.render(render_pass),
            }
        }
    }

    pub fn update(&mut self, id: i32, callback: impl FnOnce(&WindowContext, &mut EntityInfo)) {
        match self.entities.get_mut(&id) {
            None => {
                self.entities.insert(id, EntityInfo::None);
                callback(&self.ctx, self.entities.get_mut(&id).unwrap());
            }
            Some(info) => {
                callback(&self.ctx, info);
            }
        }
    }

    pub fn remove(&mut self, chunks: &mut ChunkList, id: i32) {
        if let Some(old) = self.entities.remove(&id) {
            match old {
                EntityInfo::None => {}
                EntityInfo::SingleMesh(mesh) => {
                    chunks.recycle(Some(mesh));
                }
            }
        }
    }

    #[cfg(feature = "profiling")]
    pub fn log_profile(&self) {
        println!("EntityRender:\n  - loaded: {}", self.entities.len());
    }
}
