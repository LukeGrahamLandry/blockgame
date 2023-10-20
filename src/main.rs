mod camera;
mod window;
mod chunk_mesh;
mod pos;
mod textures;


use std::mem::size_of;
use std::rc::Rc;
use glam::{Mat4, Vec2, Vec3};
use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutEntry, Buffer, RenderPipeline};
use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event::ElementState::Pressed;
use winit::window::CursorGrabMode;
use crate::camera::{CameraController, CameraHandle, SpectatorCameraController};
use crate::chunk_mesh::ChunkList;
use crate::pos::{Tile, Chunk, LocalPos, ChunkPos};
use crate::textures::TextureAtlas;
use crate::window::{App, Mesh, MeshUniform, ModelVertex, ref_to_bytes, slice_to_bytes, Texture, WindowContext};

fn main() {
    env_logger::init();
    pollster::block_on(WindowContext::run(State::new));
}

enum ControlMode {
    Fly,
    Walk
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
        let atlas = Rc::new(TextureAtlas::new(ctx.clone()));
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

        let mut chunk = Chunk::full(Tile(0));

        for i in 0..16 {
            for j in 0..16 {
                chunk.set(LocalPos::new(j, 0, i), Tile(1));
            }
        }
        for i in 0..8 {
            chunk.set(LocalPos::new(0, 1, i), Tile(1));
        }

        let mut chunks = ChunkList::new(ctx.clone(), atlas.clone(), info_bind_group_layout);
        chunks.update_mesh(ChunkPos::new(0, 0, 0), &chunk);
        chunks.update_mesh(ChunkPos::new(0, 0, 1), &chunk);
        chunks.update_mesh(ChunkPos::new(0, 0, 2), &chunk);
        for i in 0..16 {
            for j in 0..16 {
                chunk.set(LocalPos::new(j, 2, i), Tile(1));
            }
        }
        chunks.update_mesh(ChunkPos::new(1, 1, 1), &chunk);

        chunk = Chunk::full(Tile(1));
        chunks.update_mesh(ChunkPos::new(0, 1, 2), &chunk);

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

        match event {
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                },

                ..
            } => {
                match key {
                    VirtualKeyCode::Key1 => self.controller.speed /= 2.0,
                    VirtualKeyCode::Key2 => self.controller.speed *= 2.0,
                    VirtualKeyCode::Tab => if *state == Pressed {
                        self.cursor_lock = !self.cursor_lock;
                        if self.cursor_lock {
                            self.ctx.window.set_cursor_grab(CursorGrabMode::Locked);
                            self.ctx.window.set_cursor_visible(false);
                        } else {
                            self.ctx.window.set_cursor_grab(CursorGrabMode::None);
                            self.ctx.window.set_cursor_visible(true);
                        }
                        self.controller.frozen = !self.controller.frozen;
                    }
                    _ => {  }
                }
            },
            _ => { }
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
