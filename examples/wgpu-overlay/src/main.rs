#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use eframe::egui;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    center_x: f32,
    center_y: f32,
    zoom: f32,

    color: f32,
    trap_x: f32,
    trap_y: f32,
    _pad: f32,
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let app = Arc::new(pollster::block_on(CustomWgpuApp::new()));

    let wgpu_configuration = {
        let app_clone = Arc::clone(&app);
        egui_wgpu::WgpuConfiguration {
            wgpu_setup: egui_wgpu::WgpuSetup::Existing(egui_wgpu::WgpuSetupExisting {
                instance: app.instance.clone(),
                adapter: app.adapter.clone(),
                device: app.device.clone(),
                queue: app.queue.clone(),
            }),
            paint_background_hook: Some(Arc::new(move |props| app_clone.paint(props))),
            ..Default::default()
        }
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        wgpu_options: wgpu_configuration,
        ..Default::default()
    };

    eframe::run_simple_native("wgpu + egui overlay demo", options, move |ctx, _frame| {
        let mut uniforms = app.uniforms.write().unwrap();
        egui::Window::new("settings").show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut uniforms.trap_x, -3.0..=3.0).text("trap_x"));
            ui.add(egui::Slider::new(&mut uniforms.trap_y, -3.0..=3.0).text("trap_y"));
            ui.add(egui::Slider::new(&mut uniforms.color, 0.0..=10.0).text("color"));
        });
        let zoom_delta = ctx.input(|i| i.smooth_scroll_delta.y);
        uniforms.zoom += zoom_delta / 1000.0;
        let primary_down = ctx.input(|i| i.pointer.primary_down());
        if primary_down {
            let zoom_factor = 2.0f32.powf(uniforms.zoom * -20.0);
            let pointer_delta = ctx.input(|i| i.pointer.delta());
            let window_size = ctx.input(|i| i.viewport().inner_rect.unwrap().size());
            let pointer_delta = pointer_delta * zoom_factor;
            uniforms.center_x += pointer_delta.x / window_size.x;
            uniforms.center_y += pointer_delta.y / window_size.y;
        }
        ctx.request_repaint();
    })
}

struct CustomWgpuApp {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,

    pub uniforms: RwLock<Uniforms>,

    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    timer: Instant,
}

impl CustomWgpuApp {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            instance,
            adapter,
            queue,
            device,
            pipeline,
            bind_group,
            uniform_buffer,
            uniforms: RwLock::new(Uniforms {
                time: 0.0,
                center_x: 0.65,
                center_y: 0.45,
                color: 0.5,
                zoom: 0.0,
                trap_x: 0.7,
                trap_y: 0.7,
                _pad: 0.0,
            }),
            timer: Instant::now(),
        }
    }

    fn paint(&self, props: egui_wgpu::PaintBackgroundProps) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut uniforms = self.uniforms.write().unwrap();
            uniforms.time = self.timer.elapsed().as_secs_f32();
            self.queue
                .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &props.surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..wgpu::RenderPassDescriptor::default()
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
    }
}
