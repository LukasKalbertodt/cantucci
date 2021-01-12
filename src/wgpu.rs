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

        let desc = swap_chain_description(window.inner_size());
        let swap_chain = device.create_swap_chain(&surface, &desc);
        debug!("Created swapchain with dimensions {}x{}", desc.width, desc.height);

        info!("Finished wgpu intialization");

        Ok(Self {
            device: Arc::new(device),
            surface,
            swap_chain,
            swap_chain_format: desc.format,
            queue,
        })
    }

    pub(crate) fn recreate_swap_chain(&mut self, new_size: PhysicalSize<u32>) {
        let desc = swap_chain_description(new_size);
        self.swap_chain = self.device.create_swap_chain(&self.surface, &desc);
    }
}

fn swap_chain_description(size: PhysicalSize<u32>) -> wgpu::SwapChainDescriptor {
    wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DrawContext<'a> {
    pub(crate) frame: &'a wgpu::SwapChainTexture,
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
}
