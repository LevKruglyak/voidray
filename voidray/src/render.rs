use crossbeam::channel::{bounded, Sender};
use rand::thread_rng;
use rand::{distributions::Uniform, prelude::Distribution};
use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};
use vulkano::{
    buffer::cpu_access::WriteLock,
    memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc},
    sync,
};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo},
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageUsage},
    sync::GpuFuture,
};
use vulkano_util::context::VulkanoContext;

use log::*;
use rayon::prelude::*;

use crate::core::scene::Scene;
use crate::core::tracer::{RenderSettings, trace_ray};
use crate::utils::color::Color;

pub enum RenderAction {
    Start,
    Cancel,
}

pub struct Renderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    currently_rendering: Arc<RwLock<bool>>,
    sample_count: Arc<RwLock<(u32, u32)>>,
    sender: Sender<RenderAction>,
}

impl Renderer {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        scene: Arc<RwLock<Scene>>,
        target: Arc<RwLock<RenderTarget>>,
        settings: Arc<RwLock<RenderSettings>>,
    ) -> Self {
        let (sender, receiver) = bounded(0);

        let sample_count = Arc::new(RwLock::new((0, 0)));
        let currently_rendering = Arc::new(RwLock::new(false));

        let thread_currently_rendering = currently_rendering.clone();
        let thread_sample_count = sample_count.clone();

        let thread_device = device.clone();
        let thread_queue = queue.clone();

        thread::spawn(move || {
            loop {
                if let Ok(RenderAction::Start) = receiver.try_recv() {
                    *thread_currently_rendering.write().unwrap() = true;
                    info!("started rendering...");

                    let settings = {
                        let settings = settings.read().unwrap();
                        settings.clone()
                    };

                    let dimensions = {
                        let mut target_write = target.write().unwrap();
                        target_write.clear();

                        *thread_sample_count.write().unwrap() = (0, settings.samples_per_pixel);
                        target_write.samples = (0, settings.samples_per_pixel);

                        target_write.dimensions
                    };

                    info!("cleared target");

                    // Build acceleration structures
                    let scene_write = scene.write().unwrap();
                    let scene_accel = scene_write.to_acceleration();

                    info!("built acceleration structures");

                    let settings_ref = &settings;
                    let render_task = move |(index, pixel): (usize, &mut [f32]),
                                       num_samples| {
                        let x = index as u32 % dimensions[0];
                        let y = index as u32 / dimensions[1];

                        // Set up rng
                        let mut rng = thread_rng();
                        let range = Uniform::from(0.0..=1.0);
                        let mut color = Color::new(0.0, 0.0, 0.0);

                        for _ in 0..num_samples {
                            // UV coordinates
                            let u =
                                (x as f32 + range.sample(&mut rng)) / (dimensions[0] - 1) as f32;
                            let v =
                                (y as f32 + range.sample(&mut rng)) / (dimensions[1] - 1) as f32;

                            color += trace_ray(&scene_accel, settings_ref, u, v);
                        }

                        color *= 1.0 / settings.samples_per_pixel as f32;

                        pixel[0] += color.x;
                        pixel[1] += color.y;
                        pixel[2] += color.z;
                    };

                    let samples_per_run = settings.samples_per_run;
                    let mut samples = 0;

                    while samples < settings.samples_per_pixel {
                        let color: f32 = rand::random();

                        {
                            let mut target_write = target.write().unwrap();

                            target_write
                                .buffer()
                                .par_chunks_exact_mut(4)
                                .enumerate()
                                .for_each(|(index, pixel)| {
                                    render_task(
                                        (index, pixel),
                                        std::cmp::min(
                                            samples_per_run,
                                            settings.samples_per_pixel - samples,
                                        ),
                                    )
                                });

                            samples = std::cmp::min(
                                samples + samples_per_run,
                                settings.samples_per_pixel,
                            );

                            *thread_sample_count.write().unwrap() = target_write.samples;
                            target_write.samples = (samples, settings.samples_per_pixel);
                            target_write.synced = false;
                        }

                        if let Ok(RenderAction::Cancel) = receiver.try_recv() {
                            break;
                        }

                        thread::sleep(Duration::from_millis(1));
                    }

                    info!("finished rendering...");

                    *thread_currently_rendering.write().unwrap() = false;
                }

                // Prevent polling all the time
                thread::sleep(Duration::from_millis(25));
            }
        });

        Self {
            device,
            queue,
            currently_rendering,
            sample_count,
            sender,
        }
    }

    pub fn execute(&self, action: RenderAction) {
        self.sender.send(action).unwrap();
    }

    pub fn currently_rendering(&self) -> bool {
        *self.currently_rendering.read().unwrap()
    }

    pub fn sample_count(&self) -> (u32, u32) {
        *self.sample_count.read().unwrap()
    }
}

pub struct RenderTarget {
    device: Arc<Device>,
    queue: Arc<Queue>,
    buffer: Arc<CpuAccessibleBuffer<[f32]>>,
    dimensions: [u32; 2],
    samples: (u32, u32),
    synced: bool,
}

pub struct RenderTargetView {
    image: Arc<AttachmentImage>,
    view: Arc<ImageView<AttachmentImage>>,
}

impl RenderTargetView {
    pub fn new(context: &VulkanoContext, dimensions: [u32; 2]) -> Self {
        let image = AttachmentImage::with_usage(
            context.device(),
            dimensions,
            Format::R32G32B32A32_SFLOAT,
            ImageUsage {
                transfer_dst: true,
                sampled: true,
                ..ImageUsage::color_attachment()
            },
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

    pub fn image(&self) -> Arc<AttachmentImage> {
        self.image.clone()
    }
}

impl RenderTarget {
    pub fn new(context: &VulkanoContext, dimensions: [u32; 2]) -> Self {
        let buffer = CpuAccessibleBuffer::from_iter(
            context.device(),
            BufferUsage::transfer_src(),
            false,
            (0..dimensions[0] * dimensions[1] * 4).map(|i| 0.0),
        )
        .unwrap();

        Self {
            device: context.device(),
            queue: context.compute_queue(),
            buffer,
            dimensions,
            samples: (0, 0),
            synced: false,
        }
    }

    pub fn copy_to_view(&mut self, view: Arc<RenderTargetView>) {
        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                self.buffer.clone(),
                view.image(),
            ))
            .unwrap();

        let command_buffer = builder.build().unwrap();
        self.synced = true;

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }

    pub fn buffer(&mut self) -> WriteLock<[f32], PotentialDedicatedAllocation<StdMemoryPoolAlloc>> {
        self.buffer.write().unwrap()
    }

    pub fn needs_sync(&self) -> bool {
        !self.synced
    }

    pub fn scale(&self) -> f32 {
        self.samples.1 as f32 / self.samples.0 as f32
    }

    pub fn clear(&mut self) {
        self.buffer().iter_mut().for_each(|x| *x = 0.0);
    }
}
