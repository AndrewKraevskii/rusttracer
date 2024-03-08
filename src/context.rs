use wgpu::DeviceDescriptor;

use wgpu::PowerPreference;

use tracing::debug;

use wgpu;

use wgpu::SurfaceConfiguration;

use winit::window::Window;

use std::sync::Arc;

use wgpu::Surface;

pub(crate) struct DrawingContext {
    pub(crate) surface: Surface<'static>,
    pub(crate) window: Arc<Window>,
    pub(crate) config: SurfaceConfiguration,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) adapter: wgpu::Adapter,
}

impl DrawingContext {
    pub(crate) async fn new(window: Window) -> anyhow::Result<Self> {
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

        let config = surface.get_default_config(&adapter, width, height).unwrap();
        surface.configure(&device, &config);
        Ok(DrawingContext {
            surface,
            window,
            config,
            device,
            queue,
            adapter,
        })
    }
}
