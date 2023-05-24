use std::mem;

use image::DynamicImage;
use threerender_traits::mesh::TextureFormat;

use crate::gltf::fetcher::{Buffer, GltfImage};

pub struct Image {
    width: u32,
    height: u32,
    format: TextureFormat,
    data: Buffer,
}

impl Image {
    // TODO: improve image format data
    pub fn from_image(img: DynamicImage) -> Self {
        let width = img.width();
        let height = img.height();
        let format = match img.color() {
            image::ColorType::Rgba8 => TextureFormat::Rgba8,
            image::ColorType::Rgba16 => TextureFormat::Rgba8,
            image::ColorType::Rgb8 => TextureFormat::Rgba8,
            image::ColorType::Rgb16 => TextureFormat::Rgba8,
            _ => unimplemented!(),
        };
        let data = match img.color() {
            image::ColorType::Rgba8 => img.into_rgba8().into_vec(),
            image::ColorType::Rgba16 => img.into_rgba8().into_vec(),
            image::ColorType::Rgb8 => img.into_rgba8().into_vec(),
            image::ColorType::Rgb16 => img.into_rgba8().into_vec(),
            _ => unimplemented!(),
        };
        Self {
            width,
            height,
            format,
            data,
        }
    }
}

impl GltfImage for Image {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn format(&self) -> TextureFormat {
        self.format.clone()
    }
    fn data(&mut self) -> Buffer {
        mem::take(&mut self.data)
    }
}
