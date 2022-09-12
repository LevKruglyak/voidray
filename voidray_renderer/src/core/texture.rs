use std::sync::Arc;

use crate::{color::Color, math::lerp, vector::Float};

pub trait AbstractTexture: Send + Sync {
    fn sample(&self, u: Float, v: Float) -> Color;
}

pub enum Texture {
    Image(ImageTexture),
    Abstract(Arc<dyn AbstractTexture>),
}

impl AbstractTexture for Texture {
    fn sample(&self, u: Float, v: Float) -> Color {
        match self {
            Texture::Image(texture) => texture.sample(u, v),
            Texture::Abstract(texture) => texture.sample(u, v),
        }
    }
}

pub enum SampleType {
    Nearest,
    Bilinear,
}

pub struct ImageTexture {
    image: Vec<Color>,
    width: usize,
    height: usize,
    sample_type: SampleType,
}

impl ImageTexture {
    pub fn new(path: &str, sample_type: SampleType) -> Self {
        let image = image::open(path).unwrap().to_rgb32f();
        let dimensions = image.dimensions();

        Self {
            image: image
                .into_vec()
                .chunks_exact(3)
                .map(|data| Color::new(data[0], data[1], data[2]))
                .collect(),
            width: dimensions.0 as usize,
            height: dimensions.1 as usize,
            sample_type,
        }
    }

    fn nearest_sample(&self, x: Float, y: Float) -> Color {
        let x = (x as usize).min(self.width - 1);
        let y = (y as usize).min(self.height - 1);

        self.image[(y * self.width + x) as usize % self.image.len()]
    }

    fn bilinear_sample(&self, x: Float, y: Float) -> Color {
        let len = self.image.len();
        let x0 = (x as usize).min(self.width - 1);
        let y0 = (y as usize).min(self.height - 1);
        let ax = x - x0 as Float;
        let ay = y - y0 as Float;
        lerp(
            lerp(
                self.image[(y0 * self.width + x0) as usize % len],
                self.image[(y0 * self.width + x0 + 1) as usize % len],
                ax,
            ),
            lerp(
                self.image[((y0 + 1) * self.width + x0) as usize % len],
                self.image[((y0 + 1) * self.width + x0 + 1) as usize % len],
                ax,
            ),
            ay,
        )
    }
}

impl AbstractTexture for ImageTexture {
    fn sample(&self, mut u: Float, mut v: Float) -> Color {
        // if u < 0.0 { u += u.trunc() + 1.0; }
        // if v < 0.0 { v += v.trunc() + 1.0; }
        let x = ((0.2 * u + 100.0) % 1.0) * (self.width as Float);
        let y = ((0.2 * v + 100.0) % 1.0) * (self.height as Float);

        match self.sample_type {
            SampleType::Nearest => self.nearest_sample(x, y),
            SampleType::Bilinear => self.bilinear_sample(x, y),
        }
    }
}
