use std::sync::Arc;

use anyhow::Result;
use encase::ShaderType;
use tracing::debug;
use wgpu::{
    include_wgsl, BindGroupLayoutEntry, BufferDescriptor, BufferUsages, Color,
    CommandEncoderDescriptor, DeviceDescriptor, FragmentState, PipelineLayoutDescriptor,
    PowerPreference, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    ShaderStages, Surface, SurfaceConfiguration, TextureViewDescriptor, VertexState,
};
use winit::{
    error::EventLoopError,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[pollster::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let window = Window::new(&event_loop).unwrap();

    App::init(window).await?.run(event_loop).await?;
    Ok(())
}

#[derive(Debug, ShaderType)]
struct UniformState {
    time: f32,
    apsect: f32, // w/h
}

struct App {
    surface: Surface<'static>,
    window: Arc<Window>,
    config: SurfaceConfiguration,
    state: UniformState,
    device: wgpu::Device,
    queue: wgpu::Queue,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl App {
    async fn run(mut self: Self, event_loop: EventLoop<()>) -> Result<(), EventLoopError> {
        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => elwt.exit(),
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    self.config.width = size.width;
                    self.config.height = size.height;

                    self.state.apsect = self.config.width as f32 / self.config.height as f32;
                    self.surface.configure(&self.device, &self.config);
                    self.window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    self.window.request_redraw();
                    let surface_texture =
                        self.surface.get_current_texture().expect("To get texture");
                    let mut encoder = self
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor::default());
                    let view = surface_texture
                        .texture
                        .create_view(&TextureViewDescriptor::default());

                    // self.state.time = std::time::Instant::now()
                    //     .duration_since(self.start_time)
                    // .as_secs_f32();
                    self.state.time = 0.0;
                    self.queue.write_buffer(
                        &self.uniform_buffer,
                        0,
                        &self.state.as_wgsl_bytes().expect(
                            "Error in encase translating AppState \
                    struct to WGSL bytes.",
                        ),
                    );
                    {
                        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(Color::BLACK),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                        pass.set_pipeline(&self.pipeline);
                        pass.set_bind_group(0, &self.bind_group, &[]);
                        pass.draw(0..6, 0..1);
                    }
                    self.queue.submit(Some(encoder.finish()));
                    surface_texture.present();
                }
                _ => (),
            };
        })
    }

    async fn init(window: Window) -> anyhow::Result<Self> {
        let window = Arc::new(window);
        // let start_time = std::time::Instant::now();

        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        let instance = wgpu::Instance::default();
        debug!("Got instance");
        let surface = instance.create_surface(window.clone())?;
        debug!("Created surface");

        let Some(adapter) = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await
        else {
            return Err(anyhow::anyhow!("Can't get adapter"));
        };
        debug!("Got adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Device"),
                    required_features: adapter.features(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await?;

        debug!("Got device");
        let _buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 10,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<UniformState>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT | ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: Default::default(),
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            multiview: Default::default(),
        });

        let config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &config);

        let state = UniformState {
            time: 0.0,
            apsect: width as f32 / height as f32,
        };

        Ok(Self {
            state,
            surface,
            device,
            queue,
            config,
            window,
            uniform_buffer,
            bind_group,
            pipeline,
        })
    }
}

impl UniformState {
    fn as_wgsl_bytes(&self) -> encase::internal::Result<Vec<u8>> {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(self)?;
        Ok(buffer.into_inner())
    }
}
