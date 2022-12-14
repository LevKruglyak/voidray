use std::sync::{RwLock, RwLockWriteGuard};

use crate::preamble::*;
use vulkano::{
    buffer::{cpu_access::WriteLock, BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo, CopyImageInfo,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageUsage},
    memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc},
    sync::{self, GpuFuture},
};

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

    pub fn as_slice_mut(
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
    pub fn new(device: Arc<Device>, usage: ImageUsage, dimensions: [u32; 2]) -> Self {
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

    pub fn view(&self) -> Arc<ImageView<AttachmentImage>> {
        self.view.clone()
    }
}

#[derive(Clone)]
pub struct CpuRenderTarget {
    queue: Arc<Queue>,
    buffer: Arc<RwLock<CpuBufferImage>>,
    intermediate: Arc<RwLock<ViewImage>>,
    views: Arc<RwLock<Vec<ViewImage>>>,
    synced: Arc<RwLock<bool>>,
    dimensions: Arc<RwLock<[u32; 2]>>,
}

impl CpuRenderTarget {
    pub fn new(queue: Arc<Queue>, passes: u32, dimensions: [u32; 2]) -> Arc<Self> {
        assert!(passes > 0);

        let buffer = CpuBufferImage::new(queue.device().clone(), dimensions);

        let intermediate = ViewImage::new(
            queue.device().clone(),
            ImageUsage {
                transfer_src: true,
                transfer_dst: true,
                sampled: false,
                ..ImageUsage::color_attachment()
            },
            dimensions,
        );

        let mut views = Vec::new();
        for _ in 0..passes {
            views.push(ViewImage::new(
                queue.device().clone(),
                ImageUsage {
                    transfer_src: true,
                    transfer_dst: true,
                    storage: true,
                    sampled: true,
                    ..ImageUsage::color_attachment()
                },
                dimensions,
            ));
        }

        println!("created render target of dimensions {:?}", dimensions);

        Arc::new(Self {
            queue,
            buffer: Arc::new(RwLock::new(buffer)),
            intermediate: Arc::new(RwLock::new(intermediate)),
            views: Arc::new(RwLock::new(views)),
            synced: Arc::new(RwLock::new(false)),
            dimensions: Arc::new(RwLock::new(dimensions)),
        })
    }

    pub fn dimensions(&self) -> [u32; 2] {
        *self.dimensions.read().unwrap()
    }

    /// Resizes the render target
    /// - Will block on all internal locks, make sure no threads are using the buffer or view
    pub fn resize(&self, new_dimensions: [u32; 2]) {
        *self.dimensions.write().unwrap() = new_dimensions;

        *self.buffer.write().unwrap() =
            CpuBufferImage::new(self.queue.device().clone(), new_dimensions);
        *self.intermediate.write().unwrap() = ViewImage::new(
            self.queue.device().clone(),
            ImageUsage {
                transfer_src: true,
                transfer_dst: true,
                sampled: false,
                ..ImageUsage::color_attachment()
            },
            new_dimensions,
        );

        let passes = self.views.read().unwrap().len();
        let mut views = Vec::new();
        for _ in 0..passes {
            views.push(ViewImage::new(
                self.queue.device().clone(),
                ImageUsage {
                    transfer_src: true,
                    transfer_dst: true,
                    storage: true,
                    sampled: true,
                    ..ImageUsage::color_attachment()
                },
                new_dimensions,
            ));
        }
        *self.views.write().unwrap() = views;
    }

    /// Assuming we can block on the buffer, try to copy it to the intermediate view
    pub fn try_push(&self) {
        let buffer_read = self.buffer.read().unwrap();
        if let Ok(intermediate_write) = self.intermediate.try_write() {
            let mut builder = AutoCommandBufferBuilder::primary(
                self.queue.device().clone(),
                self.queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            builder
                .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                    buffer_read.buffer.clone(),
                    intermediate_write.image.clone(),
                ))
                .unwrap();
            let command_buffer = builder.build().unwrap();

            let future = sync::now(self.queue.device().clone())
                .then_execute(self.queue.clone(), command_buffer)
                .unwrap()
                .then_signal_fence_and_flush()
                .unwrap();

            future.wait(None).unwrap();

            *self.synced.write().unwrap() = false;
        }
    }

    /// Assuming we can block on the buffer, blockingly copy it to the intermediate view
    pub fn push(&self) {
        let buffer_read = self.buffer.read().unwrap();

        let intermediate_write = self.intermediate.write().unwrap();
        let mut builder = AutoCommandBufferBuilder::primary(
            self.queue.device().clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                buffer_read.buffer.clone(),
                intermediate_write.image.clone(),
            ))
            .unwrap();
        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.queue.device().clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        *self.synced.write().unwrap() = false;
    }

    /// Assuming we can block on the view, try to copy into it from the intermediate view
    pub fn try_pull(&self) {
        let view = self.views.read().unwrap()[0].view.clone();
        let image = self.views.read().unwrap()[0].image.clone();
        if let Ok(intermediate_read) = self.intermediate.try_read() {
            let mut builder = AutoCommandBufferBuilder::primary(
                self.queue.device().clone(),
                self.queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            builder
                .copy_image(CopyImageInfo::images(
                    intermediate_read.image.clone(),
                    image,
                ))
                .unwrap();
            let command_buffer = builder.build().unwrap();

            let future = sync::now(self.queue.device().clone())
                .then_execute(self.queue.clone(), command_buffer)
                .unwrap()
                .then_signal_fence_and_flush()
                .unwrap();

            future.wait(None).unwrap();
            *self.synced.write().unwrap() = true;
        }
    }

    pub fn buffer(&self) -> RwLockWriteGuard<'_, CpuBufferImage> {
        self.buffer.write().unwrap()
    }

    pub fn get_view(&self, index: usize) -> Arc<ImageView<AttachmentImage>> {
        self.views.read().unwrap()[index].view.clone()
    }

    /// Returns the image view, copying if out of sync
    pub fn pull_view(&self) -> Arc<ImageView<AttachmentImage>> {
        // Should be non blocking!!!
        if !*self.synced.read().unwrap() {
            self.try_pull();
        }

        self.get_view(0)
    }

    pub fn clear(&self) {
        self.buffer()
            .as_slice_mut()
            .iter_mut()
            .for_each(|x| *x = 0.0);
        self.try_push();
    }

    pub fn force_clear(&self) {
        self.buffer()
            .as_slice_mut()
            .iter_mut()
            .for_each(|x| *x = 0.0);
        self.push();
    }
}
