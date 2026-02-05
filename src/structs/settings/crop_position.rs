use image::DynamicImage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct CropPosition {
    pub x_percent: u32,
    pub y_percent: u32,
    pub width_percent: u32,
    pub height_percent: u32,
}

impl CropPosition {
    pub fn new(x_percent: u32, y_percent: u32, width_percent: u32, height_percent: u32) -> Self {
        Self {
            x_percent,
            y_percent,
            width_percent,
            height_percent,
        }
    }

    /// Convertir en pixels et cropper l'image
    pub fn crop_image(&self, image: &DynamicImage) -> DynamicImage {
        let (crop_x, crop_y, crop_width, crop_height) =
            self.to_pixels(image.width(), image.height());
        image.crop_imm(crop_x, crop_y, crop_width, crop_height)
    }

    /// Convertir en pixels réels
    pub fn to_pixels(&self, screen_width: u32, screen_height: u32) -> (u32, u32, u32, u32) {
        let x = (screen_width * self.x_percent) / 100;
        let y = (screen_height * self.y_percent) / 100;
        let width = (screen_width * self.width_percent) / 100;
        let height = (screen_height * self.height_percent) / 100;
        (x, y, width, height)
    }
}
