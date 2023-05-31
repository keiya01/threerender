use std::fmt::Debug;

use image::{load_from_memory, DynamicImage, ImageResult};

use crate::{mesh::TextureFormat, types::Buffer};

/// A trait to retrieve necessary image information.
pub trait Image: Debug {
    /// Texture width
    fn width(&self) -> u32;
    /// Texture height
    fn height(&self) -> u32;
    /// Texture format
    fn format(&self) -> &TextureFormat;
    /// Texture row data
    fn data(&self) -> &Buffer;
    /// Texture bytes per pixel
    fn bytes_per_pixel(&self) -> u32 {
        4
    }
}

#[derive(Debug, Clone)]
pub struct DefaultImage {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub data: Buffer,
}

impl DefaultImage {
    pub fn from_buffer(buffer: &[u8]) -> ImageResult<Self> {
        Ok(Self::from_image(load_from_memory(buffer)?))
    }

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

impl Image for DefaultImage {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn format(&self) -> &TextureFormat {
        &self.format
    }
    fn data(&self) -> &Buffer {
        &self.data
    }
}
