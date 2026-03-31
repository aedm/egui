use std::iter::once;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use egui::TexturesDelta;
use egui_wgpu::{RenderState, ScreenDescriptor, wgpu};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowAttributes, WindowId};

use crate::texture_to_image::texture_to_image;
use crate::wgpu::WAIT_TIMEOUT;

/// Process-global shared window state. Created once, reused by every
/// [`HeadfulRenderer`] in the process.
struct SharedState {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    render_state: RenderState,
    surface_width: u32,
    surface_height: u32,
}

// SAFETY: `SharedState` contains winit's `EventLoop` which is `!Send + !Sync`
// on macOS due to Cocoa threading constraints. In our usage the shared state is
// only ever created and accessed from the main thread (headful tests require
// `harness = false` which runs `main()` on the main thread). The `Mutex` is
// only used to provide interior mutability, not cross-thread access.
#[allow(unsafe_code)]
unsafe impl Send for SharedState {}
#[allow(unsafe_code)]
unsafe impl Sync for SharedState {}

static SHARED: OnceLock<Mutex<SharedState>> = OnceLock::new();

fn get_or_init_shared(title: &str, width: u32, height: u32) -> &'static Mutex<SharedState> {
    SHARED.get_or_init(|| {
        let mut event_loop = EventLoop::new().expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut creator = WindowCreator {
            title: title.to_owned(),
            width,
            height,
            window: None,
        };
        event_loop.pump_app_events(Some(Duration::from_secs(1)), &mut creator);

        let window = creator
            .window
            .expect("Failed to create window (Resumed event was not received)");

        let setup = crate::wgpu::default_wgpu_setup();
        let instance = pollster::block_on(setup.new_instance());
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("Failed to create wgpu surface");

        let render_state = pollster::block_on(egui_wgpu::RenderState::create(
            &egui_wgpu::WgpuConfiguration {
                wgpu_setup: setup,
                ..Default::default()
            },
            &instance,
            Some(&surface),
            egui_wgpu::RendererOptions::PREDICTABLE,
        ))
        .expect("Failed to create render state");

        let size = window.inner_size();
        let surf_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_state.target_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![render_state.target_format],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&render_state.device, &surf_config);

        Mutex::new(SharedState {
            event_loop,
            window,
            surface,
            render_state,
            surface_width: size.width.max(1),
            surface_height: size.height.max(1),
        })
    })
}

impl SharedState {
    fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if self.surface_width == width && self.surface_height == height {
            return;
        }
        self.surface_width = width;
        self.surface_height = height;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.render_state.target_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![self.render_state.target_format],
            desired_maximum_frame_latency: 2,
        };
        self.surface.configure(&self.render_state.device, &config);
    }
}

/// A test renderer that opens a real desktop window so you can watch the test harness
/// being controlled.
///
/// All instances within a process share a single window, event loop, and wgpu
/// render state (required on macOS which only allows one `EventLoop` per process).
///
/// Create one via [`crate::HarnessBuilder::headful`].
pub struct HeadfulRenderer {
    shared: &'static Mutex<SharedState>,
}

impl HeadfulRenderer {
    /// Create a new [`HeadfulRenderer`] that opens (or reuses) a window with
    /// the given title and size.
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        let title = title.into();
        let shared = get_or_init_shared(&title, width, height);

        // Resize / retitle the shared window for this harness.
        {
            let mut state = shared.lock().unwrap();
            let _ = state
                .window
                .request_inner_size(winit::dpi::LogicalSize::new(width, height));
            state.window.set_title(&title);

            let size = state.window.inner_size();
            state.resize(size.width, size.height);
        }

        Self { shared }
    }

    /// The window's current scale factor.
    fn scale_factor(&self) -> f32 {
        self.shared.lock().unwrap().window.scale_factor() as f32
    }
}

impl crate::TestRenderer for HeadfulRenderer {
    #[cfg(feature = "eframe")]
    fn setup_eframe(&self, cc: &mut eframe::CreationContext<'_>, frame: &mut eframe::Frame) {
        let rs = self.shared.lock().unwrap().render_state.clone();
        cc.wgpu_render_state = Some(rs.clone());
        frame.wgpu_render_state = Some(rs);
    }

    fn handle_delta(&mut self, delta: &TexturesDelta) {
        let state = self.shared.lock().unwrap();
        let mut renderer = state.render_state.renderer.write();
        for (id, image) in &delta.set {
            renderer.update_texture(
                &state.render_state.device,
                &state.render_state.queue,
                *id,
                image,
            );
        }
    }

    #[cfg(any(feature = "wgpu", feature = "snapshot"))]
    fn render(
        &mut self,
        ctx: &egui::Context,
        output: &egui::FullOutput,
    ) -> Result<image::RgbaImage, String> {
        let state = self.shared.lock().unwrap();
        let mut renderer = state.render_state.renderer.write();

        let mut encoder =
            state
                .render_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Egui Command Encoder"),
                });

        let size = ctx.content_rect().size() * ctx.pixels_per_point();
        let screen = ScreenDescriptor {
            pixels_per_point: ctx.pixels_per_point(),
            size_in_pixels: [size.x.round() as u32, size.y.round() as u32],
        };

        let tessellated = ctx.tessellate(output.shapes.clone(), ctx.pixels_per_point());

        let user_buffers = renderer.update_buffers(
            &state.render_state.device,
            &state.render_state.queue,
            &mut encoder,
            &tessellated,
            &screen,
        );

        let texture =
            state
                .render_state
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("Egui Texture"),
                    size: wgpu::Extent3d {
                        width: screen.size_in_pixels[0],
                        height: screen.size_in_pixels[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: state.render_state.target_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Egui Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                })
                .forget_lifetime();

            renderer.render(&mut pass, &tessellated, &screen);
        }

        state
            .render_state
            .queue
            .submit(user_buffers.into_iter().chain(once(encoder.finish())));

        state
            .render_state
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(WAIT_TIMEOUT),
            })
            .map_err(|err| format!("PollError: {err}"))?;

        Ok(texture_to_image(
            &state.render_state.device,
            &state.render_state.queue,
            &texture,
        ))
    }

    fn native_pixels_per_point(&self) -> Option<f32> {
        Some(self.scale_factor())
    }

    fn present(&mut self, ctx: &egui::Context, output: &egui::FullOutput) {
        let mut state = self.shared.lock().unwrap();

        // Pump winit events to keep the window responsive.
        let mut handler = EventPumper;
        state
            .event_loop
            .pump_app_events(Some(Duration::ZERO), &mut handler);

        // Handle window resize.
        let size = state.window.inner_size();
        state.resize(size.width, size.height);

        // Get the current surface texture.
        let output_frame = match state.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                let (w, h) = (state.surface_width, state.surface_height);
                state.resize(w, h);
                match state.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(err) => {
                        eprintln!("Failed to get surface texture after reconfigure: {err}");
                        return;
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to get surface texture: {err}");
                return;
            }
        };

        let target_view = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let screen = ScreenDescriptor {
            pixels_per_point: ctx.pixels_per_point(),
            size_in_pixels: [state.surface_width, state.surface_height],
        };

        let tessellated = ctx.tessellate(output.shapes.clone(), ctx.pixels_per_point());

        let mut renderer = state.render_state.renderer.write();

        let mut encoder =
            state
                .render_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Headful Present Encoder"),
                });

        let user_buffers = renderer.update_buffers(
            &state.render_state.device,
            &state.render_state.queue,
            &mut encoder,
            &tessellated,
            &screen,
        );

        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Headful Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                })
                .forget_lifetime();

            renderer.render(&mut pass, &tessellated, &screen);
        }

        // Free textures after render.
        for id in &output.textures_delta.free {
            renderer.free_texture(id);
        }

        drop(renderer);

        state
            .render_state
            .queue
            .submit(user_buffers.into_iter().chain(once(encoder.finish())));

        output_frame.present();

        state.window.request_redraw();
    }
}

/// Minimal handler just to pump winit events and keep the window alive.
struct EventPumper;

impl ApplicationHandler for EventPumper {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }
}

/// Helper to create a window via `pump_app_events`.
struct WindowCreator {
    title: String,
    width: u32,
    height: u32,
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for WindowCreator {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title(&self.title)
                .with_inner_size(winit::dpi::LogicalSize::new(self.width, self.height));
            let window = event_loop
                .create_window(attrs)
                .expect("Failed to create window");
            self.window = Some(Arc::new(window));
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Exit the pump loop once the window is created.
        if self.window.is_some() {
            event_loop.exit();
        }
    }
}
