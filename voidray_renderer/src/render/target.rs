use std::sync::RwLock;

use crate::preamble::*;
use rayon::slice::ParallelSliceMut;
use vulkano::{
    buffer::{cpu_access::WriteLock, BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo, CopyImageInfo},
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageUsage},
    memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc},
    sync::{self, GpuFuture},
};
use vulkano_util::context::VulkanoContext;

#[derive(Default)]
pub struct CpuBufferImageStats {
    samples: u32,
    total_samples: u32,
}

pub struct CpuBufferImage {
    buffer: Arc<CpuAccessibleBuffer<[Float]>>,
    stats: CpuBufferImageStats,
    dimensions: [u32; 2],
}

impl CpuBufferImage {
    pub fn new(device: Arc<Device>, dimensions: [u32; 2]) -> Self {
        // Create buffer image
        let buffer = CpuAccessibleBuffer::from_iter(
            device,
            BufferUsage::transfer_src(),
            false,
            (0..dimensions[0] * dimensions[1] * 4).map(|i| 0.0 as Float),
        )
        .unwrap();

        Self {
            buffer,
            stats: CpuBufferImageStats::default(),
            dimensions,
        }
    }

    fn as_slice_mut(
        &mut self,
    ) -> WriteLock<[Float], PotentialDedicatedAllocation<StdMemoryPoolAlloc>> {
        self.buffer.write().unwrap()
    }
}

pub struct ViewImage {
    image: Arc<AttachmentImage>,
    view: Arc<ImageView<AttachmentImage>>,
}

impl ViewImage {
    pub fn new(
        device: Arc<Device>,
        usage: ImageUsage,
        dimensions: [u32; 2],
    ) -> Self {
        let image = AttachmentImage::with_usage(
            device,
            [dimensions[0] as u32, dimensions[1] as u32],
            Format::R32G32B32A32_SFLOAT,
            usage,
        )
        .unwrap();

        Self {
            image: image.clone(),
            view: ImageView::new_default(image).unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct CpuRenderTarget {
    device: Arc<Device>,
    queue: Arc<Queue>,
    buffer: Arc<RwLock<CpuBufferImage>>,
    intermediate: Arc<RwLock<ViewImage>>,
    view: Arc<RwLock<ViewImage>>,
    synced: bool,
    dimensions: [u32; 2],
}

impl CpuRenderTarget {
    pub fn new(context: &VulkanoContext, dimensions: [u32; 2]) -> Arc<RwLock<Self>> {
        let buffer = CpuBufferImage::new(context.device(), dimensions);

        let intermediate = ViewImage::new(
            context.device(),
            ImageUsage {
                transfer_src: true,
                transfer_dst: true,
                sampled: false,
                ..ImageUsage::color_attachment()
            },
            dimensions,
        );

        let view = ViewImage::new(
            context.device(),
            ImageUsage {
                transfer_src: false,
                transfer_dst: true,
                sampled: true,
                ..ImageUsage::color_attachment()
            },
            dimensions,
        );

        Arc::new(RwLock::new(Self {
            device: context.device(),
            queue: context.compute_queue(),
            buffer: Arc::new(RwLock::new(buffer)),
            intermediate: Arc::new(RwLock::new(intermediate)),
            view: Arc::new(RwLock::new(view)),
            synced: false,
            dimensions,
        }))
    }

    /// Resizes the render target
    /// - Will block on all internal locks, make sure no threads are using the buffer or view
    pub fn resize(&mut self, new_dimensions: [u32; 2]) {
        self.dimensions = new_dimensions;

        *self.buffer.write().unwrap() = CpuBufferImage::new(self.device.clone(), new_dimensions);
        *self.intermediate.write().unwrap() = ViewImage::new(
            self.device.clone(),
            ImageUsage {
                transfer_src: true,
                transfer_dst: true,
                sampled: false,
                ..ImageUsage::color_attachment()
            },
            new_dimensions,
        );
        *self.view.write().unwrap() = ViewImage::new(
            self.device.clone(),
            ImageUsage {
                transfer_src: false,
                transfer_dst: true,
                sampled: true,
                ..ImageUsage::color_attachment()
            },
            new_dimensions,
        );

    }

    /// Assuming we can block on the buffer, try to copy it to the intermediate view
    pub fn try_push(&mut self) {
        let buffer_read = self.buffer.read().unwrap();
        if let Ok(intermediate_write) = self.intermediate.try_write() {
            let mut builder = AutoCommandBufferBuilder::primary(
                self.device.clone(),
                self.queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            builder.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                buffer_read.buffer.clone(),
                intermediate_write.image.clone(),
            ));
            let command_buffer = builder.build().unwrap();

            let future = sync::now(self.device.clone())
                .then_execute(self.queue.clone(), command_buffer)
                .unwrap()
                .then_signal_fence_and_flush()
                .unwrap();

            future.wait(None).unwrap();
            self.synced = false;
        }
    }

    /// Assuming we can block on the view, try to copy into it from the intermediate view
    pub fn try_pull(&mut self) {
        let view_write = self.view.read().unwrap();
        if let Ok(intermediate_read) = self.intermediate.try_read() {
            let mut builder = AutoCommandBufferBuilder::primary(
                self.device.clone(),
                self.queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            builder.copy_image(CopyImageInfo::images(
                intermediate_read.image.clone(),
                view_write.image.clone(),
            ));
            let command_buffer = builder.build().unwrap();

            let future = sync::now(self.device.clone())
                .then_execute(self.queue.clone(), command_buffer)
                .unwrap()
                .then_signal_fence_and_flush()
                .unwrap();

            future.wait(None).unwrap();
            self.synced = true;
        }
    }

    pub fn buffer(&self) -> Arc<RwLock<CpuBufferImage>> {
        self.buffer.clone()
    }

    /// Returns the image view, copying if out of sync
    pub fn view(&mut self) -> Arc<RwLock<ViewImage>> {
        if !self.synced {
            self.try_pull();
        }

        self.view.clone()
    }
}
