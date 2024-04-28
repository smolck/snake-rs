mod game;
mod shader;

// TODO(smolck): checkkered pattern background option
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use wgpu::util::DeviceExt;

struct Uniforms {
    window_resolution: [f32; 2],
    snake_color: [f32; 3],
    bg_color: [f32; 3],
    food_color: [f32; 3],

    colors_buffer: wgpu::Buffer,
    resolution_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Uniforms {
    pub fn new(
        device: &wgpu::Device,
        win_width: f32,
        win_height: f32,
        snake_color: [f32; 3],
        bg_color: [f32; 3],
        food_color: [f32; 3],
    ) -> Self {
        let resolution_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("resolution uniforms buffer"),
            contents: bytemuck::cast_slice(&[win_width, win_height]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let colors_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniforms buffer"),
            contents: bytemuck::cast_slice(&[
                snake_color[0],
                snake_color[1],
                snake_color[2],
                0.,
                bg_color[0],
                bg_color[1],
                bg_color[2],
                0.,
                food_color[0],
                food_color[1],
                food_color[2],
                0.,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniforms_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniforms bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: colors_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: resolution_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            window_resolution: [win_width, win_height],
            snake_color: [1., 1., 1.],
            bg_color: [0., 0., 0.],
            food_color: [1., 0., 0.],

            resolution_buffer,
            colors_buffer,
            bind_group,
            bind_group_layout,
        }
    }
}

struct State {
    game_state: game::Game,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window,

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,

    uniforms: Uniforms,
}

impl State {
    pub async fn new(window: Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12
                | wgpu::Backends::DX11
                | wgpu::Backends::VULKAN
                | wgpu::Backends::METAL
                | wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(&surface_caps.formats[0]);

        let win_size = window.inner_size();
        println!("win size: {:?}", win_size);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface_format,
            width: win_size.width,
            height: win_size.height,
            present_mode: surface_caps.present_modes[0], // TODO(smolck)
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let snake_color = [1., 1., 1.];
        let bg_color = [0., 0., 0.];
        let food_color = [1., 0., 0.];
        let uniforms = Uniforms::new(
            &device,
            win_size.width as f32,
            win_size.height as f32,
            snake_color,
            bg_color,
            food_color,
        );

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex buffer"),
            mapped_at_creation: false,
            size: 100_000_000, // TODO(smolck): yeah probably no lol
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniforms.bind_group_layout],
                push_constant_ranges: &[],
            });

        // TODO(smolck)
        let num_vertices = 6;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[shader::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,                         // no multisampling
                mask: !0,                         // no multisampling also tf does the bang do
                alpha_to_coverage_enabled: false, // apparently antialiasing something something
            },
            multiview: None, // can render to array textures whatever that means but NO
                             // also man is wgpu explicit lol
        });

        let game = game::Game::new(win_size.width as f32, win_size.height as f32, 20.0);

        Self {
            surface,
            config,
            device,
            queue,
            size: win_size,
            window,

            game_state: game,

            render_pipeline,
            vertex_buffer,
            num_vertices,

            uniforms,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            // TODO(smolck): tf
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let render_data = self.game_state.render_data();
        self.num_vertices = render_data.len() as u32;
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&render_data));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.uniforms.bind_group, &[]);
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();

    let mut state = State::new(window).await;
    event_loop.set_control_flow(ControlFlow::Poll);

    // event_loop.run_app(

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == state.window.id() => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(new_size) => state.resize(*new_size),
                /*WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size)
                }*/
                WindowEvent::KeyboardInput {
                    event: KeyEvent { physical_key, .. },
                    ..
                } => {
                    if let PhysicalKey::Code(key_code) = physical_key {
                        state.game_state.change_direction(match key_code {
                            KeyCode::ArrowUp => game::Direction::Up,
                            KeyCode::ArrowDown => game::Direction::Down,
                            KeyCode::ArrowLeft => game::Direction::Left,
                            KeyCode::ArrowRight => game::Direction::Right,
                            // TODO(smolck)
                            _ => state.game_state.current_direction(),
                        });
                    }
                }
                WindowEvent::RedrawRequested => {
                    // state.game_state.change_direction(game::Direction::Left);

                    if !state.game_state.update() {
                        // TODO(smolck): Display a lose message
                        state.game_state.reset();
                    }

                    std::thread::sleep(std::time::Duration::from_millis(100));
                    // state.game_state.update();
                    // state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }

                    state.window.request_redraw();
                }
                // WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                // match input { }
                // }
                _ => {}
            },

            /*Event::MainEventsCleared => {
                state.window.request_redraw();
            }*/
            _ => {}
        })
        .expect("failure?");
}

fn main() {
    pollster::block_on(run());
}
