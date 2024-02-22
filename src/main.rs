use tracing::debug;
use wgpu::{
    include_wgsl, BufferDescriptor, BufferUsages, Color, CommandEncoderDescriptor,
    DeviceDescriptor, FragmentState, PipelineLayoutDescriptor, PowerPreference,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    TextureViewDescriptor, VertexState,
};
use winit::{
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

    run(event_loop, window).await?;
    Ok(())
}

async fn run(event_loop: EventLoop<()>, window: Window) -> anyhow::Result<()> {
    let size = window.inner_size();
    let width = size.width;
    let height = size.height;

    let instance = wgpu::Instance::default();
    debug!("Got instance");
    let surface = instance.create_surface(&window)?;
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

    let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
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

    let mut config = surface.get_default_config(&adapter, width, height).unwrap();
    surface.configure(&device, &config);

    let window = &window;
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
                config.width = size.width;
                config.height = size.height;

                surface.configure(&device, &config);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let surface_texture = surface.get_current_texture().expect("To get texture");
                let mut encoder =
                    device.create_command_encoder(&CommandEncoderDescriptor::default());
                let view = surface_texture
                    .texture
                    .create_view(&TextureViewDescriptor::default());
                {
                    let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(Color::GREEN),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    pass.set_pipeline(&pipeline);
                    pass.draw(0..3, 0..1);
                }
                queue.submit(Some(encoder.finish()));
                surface_texture.present();
            }
            _ => (),
        };
    })?;
    Ok(())
}
