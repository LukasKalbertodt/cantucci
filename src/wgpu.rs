use std::sync::Arc;

use winit::{
    dpi::PhysicalSize,
    window::Window,
};

use crate::prelude::*;


/// Holds all important central handles to wgpu objects.
pub(crate) struct Wgpu {
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) surface: wgpu::Surface,
    pub(crate) swap_chain: wgpu::SwapChain,
    pub(crate) swap_chain_format: wgpu::TextureFormat,
    pub(crate) depth_buffer: wgpu::TextureView,
    pub(crate) queue: wgpu::Queue,
}

impl Wgpu {
    pub(crate) async fn new(window: &Window) -> Result<Self> {
        info!("Initializing wgpu...");

        // Create an instance, just a temporary object to get access to other
        // objects.
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        debug!("Created wgpu instance");

        // Create the surface, something we can draw on (or well, we will create
        // a swapchain from). The surface call is `unsafe` because we must
        // ensure `window` is a valid raw window handle to create a surface on.
        // Let's just assume it is.
        let surface = unsafe { instance.create_surface(&*window) };
        debug!("Created wgpu surface");

        // The adapter is a physical device. The variable is only temporary and
        // only used to create a "logical device" (the `device`).
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .context("Failed to find an appropiate adapter")?;

        debug!(
            "Created wgpu adapter: {} ({:?}, {:?})",
            adapter.get_info().name,
            adapter.get_info().backend,
            adapter.get_info().device_type,
        );
        trace!("Adapter features: {:#?}", adapter.features());
        trace!("Adapter limits: {:#?}", adapter.limits());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::PUSH_CONSTANTS,
                    limits: wgpu::Limits {
                        max_push_constant_size: adapter.limits().max_push_constant_size,
                        .. wgpu::Limits::default()
                    },
                    shader_validation: true,
                },
                None,
            )
            .await
            .context("Failed to create device")?;

        debug!("Created wgpu device (including a queue)");
        trace!("Device features: {:#?}", device.features());
        trace!("Device limits: {:#?}", device.limits());

        let (swap_chain, depth_buffer)
            = Self::create_output_images(&device, &surface, window.inner_size());

        info!("Finished wgpu intialization");

        Ok(Self {
            device: Arc::new(device),
            surface,
            swap_chain,
            swap_chain_format: SWAP_CHAIN_FORMAT,
            depth_buffer,
            queue,
        })
    }

    pub(crate) fn recreate_swap_chain(&mut self, new_size: PhysicalSize<u32>) {
        let (swap_chain, depth_buffer)
            = Self::create_output_images(&self.device, &self.surface, new_size);
        self.swap_chain = swap_chain;
        self.depth_buffer = depth_buffer;
    }

    fn create_output_images(
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        size: PhysicalSize<u32>,
    ) -> (wgpu::SwapChain, wgpu::TextureView) {
        let desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: SWAP_CHAIN_FORMAT,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(surface, &desc);
        debug!("Created swapchain with dimensions {}x{}", desc.width, desc.height);

        let depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Buffer"),
            size: wgpu::Extent3d {
                width: desc.width,
                height: desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        let depth_buffer = depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());
        debug!("Created depth buffer");

        (swap_chain, depth_buffer)
    }
}

const SWAP_CHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub(crate) const DEPTH_BUFFER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;


#[derive(Debug, Clone, Copy)]
pub(crate) struct DrawContext<'a> {
    pub(crate) frame: &'a wgpu::SwapChainTexture,
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) depth_buffer: &'a wgpu::TextureView,
}
