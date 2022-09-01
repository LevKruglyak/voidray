use std::sync::Arc;

use vulkano::{
    format::Format,
    image::{view::ImageView, ImageDimensions, StorageImage},
};
use vulkano_util::context::VulkanoContext;

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
