use std::f32::consts::FRAC_PI_2;
use std::time::Instant;
use glam::{Mat4, Vec3, Vec4};
use wgpu::VertexAttribute;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceEvent, ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use crate::window::{ref_to_bytes, WindowContext};

pub struct CameraHandle {
    pub camera: CameraPerspective,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
}

/// A projection that can be used when rendering the world.
#[derive(Debug)]
pub struct CameraPerspective {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub scale: f32
}

/// Raw form of a CameraPerspective to be sent to the GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct RawCamera {
    view_proj: [[f32; 4]; 4],
    view_pos: [f32; 4]
}

/// A strategy for moving a CameraPerspective based on user input.
pub trait CameraController {
    fn process_scroll(&mut self, delta: &MouseScrollDelta);
    fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64);
    fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool;
    fn set_mouse_pressed(&mut self, pressed: bool);

    /// Adjusts the CameraPerspective based on any user input received this frame.
    fn update_camera(&mut self, camera: &mut CameraPerspective);

    fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
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
                self.process_keyboard(*key, *state);
            },
            WindowEvent::MouseWheel { delta, .. } => {
                self.process_scroll(delta);
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.set_mouse_pressed(*state == ElementState::Pressed);
            }
            _ => {},
        };
        false
    }

    fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.process_mouse(delta.0, delta.1);
        }
    }

    fn update(&mut self, ctx: &WindowContext, camera: &mut CameraHandle) {
        self.update_camera(&mut camera.camera);
        ctx.write_buffer(&camera.camera_buffer, ref_to_bytes(&camera.camera.as_raw()));
    }

    fn resize(&mut self, camera: &mut CameraHandle, new_size: &PhysicalSize<u32>) {
        camera.camera.resize(new_size.width, new_size.height);
    }
}

impl CameraHandle {
    pub fn new(ctx: &WindowContext) -> CameraHandle {
        let camera_bind_group_layout = ctx.bind_group_layout_buffer("Camera", &[
            (wgpu::ShaderStages::VERTEX_FRAGMENT, wgpu::BufferBindingType::Uniform)
        ]);

        let mut camera = CameraPerspective::new();
        camera.resize(ctx.size.borrow().width, ctx.size.borrow().height);

        let camera_buffer = ctx.buffer_init(
            "Camera", ref_to_bytes(&camera.as_raw()),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        );

        let camera_bind_group = ctx.bind_group("camera", &camera_bind_group_layout, &[
            camera_buffer.as_entire_binding()
        ]);

        CameraHandle {
            camera,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        }
    }
}

impl CameraPerspective {
    const ATTRIBS: Option<&'static [VertexAttribute]> = Some(&wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4]);
    pub(crate) fn as_raw(&self) -> RawCamera {
        RawCamera {
            view_pos: [self.pos.x, self.pos.y, self.pos.z, 1.0],
            view_proj: self.calc_matrix().to_cols_array_2d()
        }
    }

    pub fn new() -> CameraPerspective {
        CameraPerspective {
            pos: Vec3::new(0.0, 5.0, 10.0),
            yaw: -90.0_f32.to_radians(),
            pitch: -20.0_f32.to_radians(),
            aspect: 1.0,  // set by resize()
            fovy: 45.0_f32.to_radians(),
            znear: 0.1,
            zfar: 100.0,
            scale: 1.0,
        }
    }

    pub fn calc_matrix(&self) -> Mat4 {
        let player = Mat4::look_at_rh(
            self.pos,
            (self.pos + self.facing()),
            Vec3::new(0.0, 1.0, 0.0),
        );

        let projection = OPENGL_TO_WGPU_MATRIX.transpose() * Mat4::from_scale(Vec3::new(self.scale, self.scale, self.scale)) * Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);

        projection * player
    }

    pub fn facing(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw)
    }

    pub fn position(&self) -> Vec3 {
        self.pos
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
              Vec4::new(0.0, 0.0, 0.5, 0.0),
                        Vec4::new(0.0, 0.0, 0.5, 1.0),
);

pub const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;


#[derive(Debug)]
pub struct SpectatorCameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    pub speed: f32,
    sensitivity: f32,
    control_held: bool,
    mouse_pressed: bool,
    last_update: Instant,
    pub frozen: bool
}

impl SpectatorCameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            control_held: false,
            mouse_pressed: false,
            last_update: Instant::now(),
            frozen: false,
        }
    }
}

impl CameraController for SpectatorCameraController {
    fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        if self.frozen { return; }
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * 0.5,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => -*scroll as f32,
        };
    }

    fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        if self.frozen { return; }
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        if self.frozen { return false; }
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            },
            VirtualKeyCode::LControl => {
                self.control_held = state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    fn set_mouse_pressed(&mut self, pressed: bool) {
        if self.frozen { return; }
        self.mouse_pressed = pressed;
    }

    fn update_camera(&mut self, camera: &mut CameraPerspective) {
        let dt = Instant::now() - self.last_update;
        self.last_update = Instant::now();

        let dt = dt.as_secs_f32();
        let move_speed = self.speed * if self.control_held { 5.0 } else { 1.0 };

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.pos += forward * (self.amount_forward - self.amount_backward) * move_speed * dt;
        camera.pos += right * (self.amount_right - self.amount_left) * move_speed * dt;

        camera.scale += self.scroll / 5.0 * camera.scale * dt;
        camera.scale = camera.scale.clamp(0.5, 50.0);
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.pos.y += (self.amount_up - self.amount_down) * move_speed * dt;

        // Rotate
        camera.yaw += self.rotate_horizontal * self.sensitivity * dt;
        camera.pitch += -self.rotate_vertical * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        camera.pitch = camera.pitch.clamp(-SAFE_FRAC_PI_2, SAFE_FRAC_PI_2);
    }
}
