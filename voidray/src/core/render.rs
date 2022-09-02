use std::{
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread,
    time::Duration,
};

use crossbeam::channel::{bounded, Sender};
use vulkano::{
    format::Format,
    image::{view::ImageView, ImageDimensions, StorageImage},
};
use vulkano_util::context::VulkanoContext;

use super::scene::Scene;
use log::*;

pub enum RenderAction {
    Start,
    Cancel,
}

pub struct Renderer {
    currently_rendering: Arc<RwLock<bool>>,
    sample_count: Arc<RwLock<(u32, u32)>>,
    sender: Sender<RenderAction>,
}

impl Renderer {
    pub fn new(
        scene: Arc<RwLock<Scene>>,
        target: Arc<RwLock<RenderTarget>>,
        settings: Arc<RwLock<RenderSettings>>,
    ) -> Self {
        let (sender, receiver) = bounded(0);

        let sample_count = Arc::new(RwLock::new((0, 0)));
        let currently_rendering = Arc::new(RwLock::new(false));

        let thread_sample_count = sample_count.clone();
        let thread_currently_rendering = currently_rendering.clone();

        thread::spawn(move || {
            loop {
                if let Ok(RenderAction::Start) = receiver.try_recv() {
                    *thread_currently_rendering.write().unwrap() = true;

                    let settings = {
                        let settings = settings.read().unwrap();
                        settings.clone()
                    };

                    // Perform the render
                    if settings.poll_for_canel {
                        for sample in 0..=settings.samples_per_pixel {
                            if let Ok(RenderAction::Cancel) = receiver.try_recv() {
                                break;
                            }

                            thread::sleep(Duration::from_millis(1));
                            *thread_sample_count.write().unwrap() =
                                (sample, settings.samples_per_pixel);
                        }
                    } else {
                        thread::sleep(Duration::from_millis(settings.samples_per_pixel as u64));
                    }

                    *thread_currently_rendering.write().unwrap() = false;
                }

                // Prevent polling all the time
                thread::sleep(Duration::from_millis(25));
            }
        });

        Self {
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
    image: Arc<StorageImage>,
    view: Arc<ImageView<StorageImage>>,
}

impl RenderTarget {
    pub fn new(context: &VulkanoContext, dimensions: [u32; 2]) -> Self {
        let image = StorageImage::new(
            context.device(),
            ImageDimensions::Dim2d {
                width: dimensions[0],
                height: dimensions[1],
                array_layers: 1,
            },
            Format::R8G8B8A8_UNORM,
            Some(context.graphics_queue().family()),
        )
        .unwrap();

        Self {
            image: image.clone(),
            view: ImageView::new_default(image).unwrap(),
        }
    }

    pub fn view(&self) -> Arc<ImageView<StorageImage>> {
        self.view.clone()
    }
}

pub fn render(
    scene: RwLockReadGuard<Scene>,
    target: RwLockWriteGuard<RenderTarget>,
    settings: RenderSettings,
) {
    info!("rendering...");
    info!("{:?}", settings);
    thread::sleep(Duration::from_millis(settings.sleep_duration as u64));
    info!("done rendering!");
}

#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub samples_per_pixel: u32,
    pub sleep_duration: u32,
    pub poll_for_canel: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            samples_per_pixel: 100,
            sleep_duration: 1000,
            poll_for_canel: true,
        }
    }
}
