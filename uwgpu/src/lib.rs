#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use state::State;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
mod wasm_utils;
#[cfg(target_arch = "wasm32")]
pub use wasm_utils::*;

// Canvas size (300x400) -> Output bitmat (500x500) image changes?

/// Experimenting with wgpu tutorial
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    init_logger();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    wasm_add_canvas_to_html(&window);

    let mut state = State::new(&window).await;

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            println!("Resize: {:?}", physical_size);
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => {
                                    state.resize(state.size)
                                }
                                // The system is out of memory, we should
                                // probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    eprintln!("System ran out of memory");
                                    control_flow.exit()
                                }
                                // All other errors (Outdated, Timeout) should
                                // be resolved by the next frame
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key:
                                        PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                // RedrawRequested will only trigger once unless we manually
                // request it.
                state.window().request_redraw();
            }

            _ => {}
        })
        .unwrap();
}

/// We have to do a bit of setup to enable logging of panics.
///
/// "When wgpu hits any error, it panics with a generic message, while logging
/// the real error via the log crate. This means if you don't include
/// env_logger::init(), wgpu will fail silently, leaving you very confused!"
/// Reference: https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/#env-logger
fn init_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        } else {
            env_logger::init();
        }
    }
}

mod state {
    use winit::{event::WindowEvent, window::Window};

    pub struct State<'a> {
        surface: wgpu::Surface<'a>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_config: wgpu::SurfaceConfiguration,
        pub size: winit::dpi::PhysicalSize<u32>,
        // The window must be declared after the surface so
        // it gets dropped after it as the surface contains
        // unsafe references to the window's resources.
        window: &'a Window,
        render_pipeline: wgpu::RenderPipeline,
    }

    impl<'a> State<'a> {
        // Creating some of the wgpu types requires async code
        pub async fn new(window: &'a Window) -> State<'a> {
            let size = window.inner_size();

            // The instance is a handle to our GPU
            // Backends::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            });

            let surface = instance.create_surface(window).unwrap();

            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap();

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        memory_hints: wgpu::MemoryHints::Performance,
                    },
                    None, // Trace path
                )
                .await
                .unwrap();

            let surface_caps = surface.get_capabilities(&adapter);
            // Shader code in this tutorial assumes an sRGB surface texture.
            // Using a different one will result in all the colors
            // coming out darker. If you want to support non
            // sRGB surfaces, you'll need to account for that when drawing to
            // the frame.
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);
            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            let shader =
                device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

            let render_pipeline_layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                },
            );

            let render_pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        // Vertices declared inside shader, no buffers as of yet
                        buffers: &[],
                        // Advanced options, default is fine
                        compilation_options:
                            wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            // format same as surface for easy copying
                            format: surface_config.format,
                            // just replace old data with new data
                            blend: Some(wgpu::BlendState::REPLACE),
                            // Use all colors: R,G,B,A
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options:
                            wgpu::PipelineCompilationOptions::default(),
                    }),
                    // how to interpret vertices when turning them into tringles
                    primitive: wgpu::PrimitiveState {
                        // Every 3 vertices = 1 tringle
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        // Applies to "LineStrip" and "TriangleStrip"
                        // topologies, ignore
                        strip_index_format: None,
                        // Front face of tringle, clock-wise or
                        // (current/default) counter-clockwise
                        front_face: wgpu::FrontFace::Ccw,
                        // Cull tringles facing back or facing front
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires
                        // Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    // Doesn't apply because not using depth/stencil buffers yet
                    depth_stencil: None,
                    // Multisampling / Anti-aliasing
                    multisample: wgpu::MultisampleState {
                        // How many samples per pixel (1 = no multisampling)
                        count: 1,
                        // Bitmask restricting samples, (!0 = No restriction)
                        mask: !0,
                        // Anti-aliasing, not using this
                        alpha_to_coverage_enabled: false,
                    },
                    // Only applies when rendering to array textures, we aren't
                    multiview: None,
                    // No pipeline cache
                    cache: None,
                },
            );

            Self {
                window,
                surface,
                device,
                queue,
                surface_config,
                size,
                render_pipeline,
            }
        }

        pub fn window(&self) -> &Window { &self.window }

        pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
            if new_size.width > 0 && new_size.height > 0 {
                self.size = new_size;
                self.surface_config.width = new_size.width;
                self.surface_config.height = new_size.height;

                #[cfg(target_arch = "wasm32")]
                {
                    self.surface_config.width = 500;
                    self.surface_config.height = 500;
                }
                self.surface.configure(&self.device, &self.surface_config);
            }
        }

        pub fn input(&mut self, event: &WindowEvent) -> bool { false }

        pub fn update(&mut self) { //TODO
        }

        pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
            let output = self.surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                },
            );

            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);

            // must be dropped before calling encoder.finish()
            // Reference: https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#render
            drop(render_pass);

            // submit will accept anything that implements IntoIter
            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();

            Ok(())
        }
    }
}
