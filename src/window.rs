use std::cell::RefCell;
use std::rc::Rc;
use std::slice;
use std::mem::size_of;
use instant::Instant;
use image::{GenericImageView};

use wgpu::PresentMode;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window, WindowBuilder};
use wgpu::*;
use wgpu::util::DeviceExt;


pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub transform: MeshUniform,
    pub(crate) info_buffer: Buffer,
    pub(crate) info_bind_group: BindGroup,
}

impl Mesh {
    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_bind_group(1, &self.info_bind_group, &[]);
        render_pass.draw_indexed(0..self.num_elements, 0, 0..1);
    }
}


#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct MeshUniform {
    pub transform: [[f32; 4]; 4]
}

pub struct WindowContext {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: RefCell<wgpu::SurfaceConfiguration>,
    pub size: RefCell<PhysicalSize<u32>>,
    pub window: Window,
    pub timer: RefCell<FrameTimer>,
}

pub struct FrameTimer {
    pub frame_count: i32,
    pub micro_seconds: u128,
    pub last: Instant,
}

impl FrameTimer {
    pub fn new() -> Self {
        FrameTimer {
            frame_count: 0,
            micro_seconds: 0,
            last: Instant::now(),
        }
    }

    pub fn update(&mut self){
        let now = Instant::now();
        self.micro_seconds += self.last.elapsed().as_micros();
        self.last = now;
        self.frame_count += 1;

        if self.micro_seconds > 5000000 {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        let seconds = self.micro_seconds as f64 / 1000000.0;
        let frame_time_ms = (self.micro_seconds as f64 / self.frame_count as f64).round() / 1000.0;
        let fps = self.frame_count as f64 / seconds;
        println!("{} seconds; {} frames; {} fps; {} ms per frame;", seconds, self.frame_count, fps.round(), frame_time_ms);
        self.micro_seconds = 0;
        self.frame_count = 0;
    }
}

pub trait App {
    fn new(ctx: Rc<WindowContext>) -> Self;
    /// Return false to catch the event and prevent further handler (ie. make ESC not quit the window)
    fn handle_window_event(&mut self, event: &WindowEvent) -> bool;
    // winit::event::WindowEvent::CursorMoved warns against using it for 3D camera control so now we have two event callbacks
    fn handle_device_event(&mut self, event: &DeviceEvent);
    /// Just called before render. But there's a semantic separation and I think it's nice to put it in the same impl block.
    fn update(&mut self);
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
    fn resize(&mut self, new_size: PhysicalSize<u32>);
}

impl WindowContext {
    async fn new() -> (Rc<WindowContext>, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let mut window = WindowBuilder::new().build(&event_loop).unwrap();

        platform_setup(&mut window);
        window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
        window.set_cursor_visible(false);

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: Default::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let mut limits = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };
        limits.max_bindings_per_bind_group = 640;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits,
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .copied().find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        (Rc::new(WindowContext {
            window,
            surface,
            device,
            queue,
            config: RefCell::new(config),
            size: RefCell::new(size),
            timer: RefCell::new(FrameTimer::new())
        }), event_loop)
    }

    pub async fn run<A, F>(constructor: F)
        where A: App + 'static, F: FnOnce(Rc<WindowContext>) -> A
    {
        let (ctx, event_loop) = WindowContext::new().await;
        println!("Initializing...");
        let start = Instant::now();
        let mut app = constructor(ctx.clone());
        let mut vsync_on = true;
        let end = Instant::now();
        println!("Initialized in {} ms; present_mode={:?}", (end - start).as_millis(), ctx.config.borrow().present_mode);
        event_loop.run(move |event, _, control_flow| match event {
            Event::DeviceEvent { ref event, .. } => { app.handle_device_event(event); }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == ctx.window.id() => if !app.handle_window_event(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        app.resize(*physical_size)
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        app.resize(**new_inner_size)
                    }
                    WindowEvent::KeyboardInput {
                        input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::V),
                            ..
                        },
                        ..
                    } => {
                        vsync_on = !vsync_on;
                        ctx.config.borrow_mut().present_mode = if vsync_on { PresentMode::AutoVsync } else { PresentMode::AutoNoVsync };
                        let size = { *ctx.size.borrow() };
                        ctx.resize(&size);
                        ctx.timer.borrow_mut().reset();
                        println!("set present_mode={:?}", ctx.config.borrow().present_mode);
                    }
                    _ => {}
                }
            },
            Event::RedrawRequested(window_id) if window_id == ctx.window.id() => {
                app.update();
                match app.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        let size = *ctx.size.borrow();
                        app.resize(size);
                    },
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
                ctx.timer.borrow_mut().update();
            }
            Event::MainEventsCleared => {
                ctx.window.request_redraw();
            }
            _ => {}
        });
    }

    pub fn resize(&self, new_size: &PhysicalSize<u32>) -> bool {
        if new_size.width > 0 && new_size.height > 0 {
            *self.size.borrow_mut() = *new_size;
            let mut config = self.config.borrow_mut();
            config.width = new_size.width;
            config.height = new_size.height;
            self.surface.configure(&self.device, &config);
            true
        } else {
            false
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn platform_setup(window: &mut Window){
    use winit::dpi::LogicalSize;
    let w = web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap() as u32;
    let h = web_sys::window().unwrap().inner_height().unwrap().as_f64().unwrap() as u32;
    window.set_inner_size(LogicalSize::new(w, h));

    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("game")?;
            let canvas = web_sys::Element::from(window.canvas());
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
}

#[cfg(not(target_arch = "wasm32"))]
fn platform_setup(_window: &mut Window){
    // NO OP
}

impl WindowContext {
    pub fn write_buffer(&self, buffer: &Buffer, data: &[u8]) {
        self.queue.write_buffer(buffer, 0, data);
    }

    pub fn buffer_init(&self, label: &str, data: &[u8], usage: BufferUsages) -> Buffer {
        self.device.create_buffer_init(
            &util::BufferInitDescriptor {
                label: Some(&*concat(label, "Buffer")),
                contents: data,
                usage,
            }
        )
    }

    pub fn bind_group_layout_buffer(&self, label: &str, entries: &[(ShaderStages, BufferBindingType)]) -> BindGroupLayout {
        let entries: Vec<_> = entries.iter().enumerate().map(|(i, (visibility, ty))| {
            BindGroupLayoutEntry {
                binding: i as u32,
                visibility: *visibility,
                ty: BindingType::Buffer {
                    ty: *ty,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        }).collect();

        self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: entries.as_slice(),
            label: Some(&*concat(label, "Bind Group Layout")),
        })
    }

    pub fn bind_group(&self, label: &str, layout: &BindGroupLayout, entries: &[BindingResource]) -> BindGroup {
        let entries: Vec<_> = entries.iter().enumerate().map(|(i, e)| {
            BindGroupEntry {
                binding: i as u32,
                resource: e.clone(),
            }
        }).collect();

        self.device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: entries.as_slice(),
            label: Some(&*concat(label, "Bind Group")),
        })
    }

    pub fn bind_group_layout_texture(&self) -> BindGroupLayout {
        self.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },  // same
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),   // as here
                    count: None,
                },
                // TODO: why was this here?
                // BindGroupLayoutEntry {
                //     binding: 2,
                //     visibility: ShaderStages::FRAGMENT,
                //     ty: BindingType::Buffer {
                //         ty: BufferBindingType::Uniform,
                //         has_dynamic_offset: false,
                //         min_binding_size: None,
                //     },
                //     count: None,
                // }
            ],
            label: Some("texture_bind_group_layout"),
        })
    }

    pub fn bind_group_texture(&self, layout: &BindGroupLayout, texture: &Texture) -> BindGroup {
        self.bind_group("texture", layout, &[
            wgpu::BindingResource::TextureView(&texture.view),
            wgpu::BindingResource::Sampler(&texture.sampler)
        ])
    }

    // This could go right in create_render_pipeline but maybe its good to let you reuse layouts.
    pub fn pipeline_layout(&self, bind_group_layouts: &[&BindGroupLayout]) -> PipelineLayout{
        self.device.create_pipeline_layout(
            &PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[],
            }
        )
    }

    pub fn render_pipeline(&self, label: &str, layout: &PipelineLayout, vertex_layouts: &[VertexBufferLayout], shader: &str) -> RenderPipeline {
        let shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&*concat(label, "Render Shader")),
            source: ShaderSource::Wgsl(shader.into()),
        });
        self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&*concat(label, "Render Pipeline")),
            layout: Some(layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_layouts,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: self.config.borrow().format,
                    // TODO: is this slower than REPLACE? Is it worth having two pipelines where one does things that I know don't have transparency and then you draw the rest on top?
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent::OVER,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,// Some(Face::Back),  // TODO: currently rendering both sides of triangles
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                // TODO: before trying the big transparency thing. try with this. browser says: Multisample count (1) must be > 1 when alphaToCoverage is enabled.
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }

    pub fn command_encoder(&self, label: &str) -> CommandEncoder {
        self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some(&*concat(label, "Command Encoder")),
        })
    }

    pub fn render_pass<'f>(&self, encoder: &'f mut CommandEncoder, screen_texture: &'f TextureView, depth_texture: &'f TextureView) -> RenderPass<'f> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: screen_texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }
                        ),
                        store: StoreOp::Store,
                    }
                })
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_texture,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
}

fn concat<'a>(a: &'a str, b: &'a str) -> String {
    let s = String::from(a) + " " + b;
    s
}


pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub(crate) fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, img: &image::DynamicImage, label: Option<&str> ) -> Self {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,  // Sharp pixels when up-scaling low resolution textures.
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self { texture, view, sampler }
    }

    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual),
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }
}

pub fn ref_to_bytes<T>(p: &T) -> &[u8] {
    unsafe {
        slice::from_raw_parts(
            (p as *const T) as *const u8,
            size_of::<T>(),
        )
    }
}

pub fn slice_to_bytes<T>(p: &[T]) -> &[u8] {
    unsafe {
        slice::from_raw_parts(
            (p as *const [T]) as *const u8,
            size_of::<T>() * p.len(),
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ModelVertex {
    pub position: [f32; 4],  // 4th is ignored (not even used as w in shader!)
    pub uv: [f32; 2]
}

impl ModelVertex {
    pub const ATTRIBS: &'static [VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2];
}
