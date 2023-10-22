mod camera;
pub mod window;
mod chunk_mesh;
mod gen;

use std::mem::size_of;
use std::rc::Rc;
use wgpu::RenderPipeline;
use winit::event::{DeviceEvent, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event::ElementState::Pressed;
use winit::window::CursorGrabMode;
use crate::camera::{CameraController, CameraHandle, SpectatorCameraController};
use crate::chunk_mesh::{ChunkList, TextureAtlas};
use common::pos::{Chunk, ChunkPos, LocalPos, Tile};
use crate::window::{App, ModelVertex, Texture, WindowContext};
use common;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch="wasm32")]
#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run(){
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

    WindowContext::run(State::new).await;
}

pub struct State {
    ctx: Rc<WindowContext>,
    depth_texture: Texture,
    camera: CameraHandle,
    render_pipeline: RenderPipeline,
    chunks: ChunkList,
    controller: SpectatorCameraController,
    atlas: Rc<TextureAtlas>,
    cursor_lock: bool
}

impl App for State {
    fn new(ctx: Rc<WindowContext>) -> Self {
        let atlas = Rc::new(TextureAtlas::new(&ctx));
        let depth_texture = Texture::create_depth_texture(&ctx.device, &ctx.config.borrow(), "depth_texture");
        let camera = CameraHandle::new(&ctx);

        let info_bind_group_layout = ctx.bind_group_layout_buffer("mesh_info", &[
            (wgpu::ShaderStages::VERTEX_FRAGMENT, wgpu::BufferBindingType::Uniform)
        ]);

        let render_pipeline_layout = ctx.pipeline_layout(&[
            &camera.camera_bind_group_layout,
            &info_bind_group_layout,
            &atlas.layout
        ]);

        let render_pipeline = ctx.render_pipeline(
            "main", &render_pipeline_layout, &[wgpu::VertexBufferLayout {
                array_stride: size_of::<ModelVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: ModelVertex::ATTRIBS
            }], include_str!("shader.wgsl")
        );

        let mut chunks = ChunkList::new(ctx.clone(), atlas.clone(), info_bind_group_layout);

        init_world(&mut chunks);

        State {
            ctx,
            depth_texture,
            camera,
            render_pipeline,
            chunks,
            controller: SpectatorCameraController::new(30.0, 0.4),
            atlas,
            cursor_lock: true,
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.controller.handle_window_event(event);

        if let WindowEvent::KeyboardInput {
            input:
            KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            },

            ..
        } = event {
            match key {
                VirtualKeyCode::Key1 => self.controller.speed /= 2.0,
                VirtualKeyCode::Key2 => self.controller.speed *= 2.0,
                VirtualKeyCode::Tab => if *state == Pressed {
                    self.cursor_lock = !self.cursor_lock;
                    if self.cursor_lock {
                        let _ = self.ctx.window.set_cursor_grab(CursorGrabMode::Locked);
                        self.ctx.window.set_cursor_visible(false);
                    } else {
                        let _ = self.ctx.window.set_cursor_grab(CursorGrabMode::None);
                        self.ctx.window.set_cursor_visible(true);
                    }
                    self.controller.frozen = !self.controller.frozen;
                }
                _ => {  }
            }
        };

        false
    }

    fn handle_device_event(&mut self, event: &DeviceEvent) {
        self.controller.handle_device_event(event);
    }

    fn update(&mut self) {
        self.controller.update(&self.ctx, &mut self.camera);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.ctx.command_encoder("render");

        let output = self.ctx.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = self.ctx.render_pass(&mut encoder, &view, &self.depth_texture.view);
            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.camera.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.atlas.bind_group, &[]);
            self.chunks.render(&mut render_pass);

        };

        self.ctx.queue.submit([
            encoder.finish()
        ]);

        output.present();

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if self.ctx.resize(&new_size) {
            self.depth_texture = Texture::create_depth_texture(&self.ctx.device, &self.ctx.config.borrow(), "depth_texture");
            self.camera.camera.resize(new_size.width, new_size.height);
        }
    }
}

// Just fill it with a bunch of random stuff for debugging.
fn init_world(chunks: &mut ChunkList) {
    let mut chunk = Chunk::full(Tile::EMPTY);
    chunks.update_mesh(ChunkPos::new(0, 2, 0), &chunk);

    for i in 0..16 {
        for j in 0..16 {
            chunk.set(LocalPos::new(j, 0, i), gen::tiles::stone);
        }
    }
    for i in 0..8 {
        chunk.set(LocalPos::new(0, 1, i), gen::tiles::dirt);
    }

    chunks.update_mesh(ChunkPos::new(0, 0, 0), &chunk);
    chunks.update_mesh(ChunkPos::new(0, 0, 1), &chunk);
    chunks.update_mesh(ChunkPos::new(0, 0, 2), &chunk);
    for i in 0..16 {
        for j in 0..16 {
            chunk.set(LocalPos::new(j, 2, i), gen::tiles::grass);
        }
    }
    chunks.update_mesh(ChunkPos::new(1, 1, 1), &chunk);

    chunk = Chunk::full(gen::tiles::stone);
    chunks.update_mesh(ChunkPos::new(0, 1, 2), &chunk);

    chunk = Chunk::full(Tile::EMPTY);
    let solids = [
        gen::tiles::stone, gen::tiles::dirt,gen::tiles::grass,gen::tiles::log,gen::tiles::leaf,gen::tiles::wheat_solid, gen::tiles::sapling_solid
    ];
    for (b, tile) in solids.iter().enumerate() {
        for i in 0..16 {
            for j in 0..(16-b) {
                chunk.set(LocalPos::new(j, b, i), *tile);
            }
        }
    }
    chunks.update_mesh(ChunkPos::new(0, 0, -1), &chunk);

    chunk = Chunk::full(Tile::EMPTY);
    let custom = [
        gen::tiles::sapling, gen::tiles::wheat,
    ];
    for (b, tile) in custom.iter().enumerate() {
        for i in 0..16 {
            for j in 0..(16-b) {
                chunk.set(LocalPos::new(j, b, i), *tile);
            }
        }
    }
    chunks.update_mesh(ChunkPos::new(-1, 0, -1), &chunk);
}
