use std::{sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}, future::Future, thread};

use vulkano::{
    format::Format,
    image::{view::ImageView, ImageDimensions, StorageImage},
};
use vulkano_util::context::VulkanoContext;

use super::scene::Scene;
use log::*;

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

pub fn render(scene: RwLockReadGuard<Scene>, target: RwLockWriteGuard<RenderTarget>, settings: RenderSettings) {
    info!("rendering...");
    info!("{:?}", settings);
    thread::sleep(std::time::Duration::from_millis(settings.sleep_duration as u64));
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
