// src/utils/screen_capture.rs
use image::DynamicImage;
use image::RgbaImage;
use xcap::Monitor;

use crate::structs::settings::crop_position::CropPosition;

/// Capture l'écran complet (sans crop)
pub async fn capture_full_screen(screen: i8) -> Result<DynamicImage, String> {
    let (dyn_image, _w, _h) = tokio::task::spawn_blocking(move || capture_screen(screen))
        .await
        .map_err(|e| format!("Erreur join: {}", e))?
        .map_err(|e| format!("Erreur capture: {}", e))?;

    #[cfg(feature = "debug")]
    {
        let _t = std::time::Instant::now();
        dyn_image.save("full_screen.png").ok();
        lap!(_t, "Save crop (disk)");
    }

    Ok(dyn_image)
}

/// Capture l'écran spécifié (fonction de base)
fn capture_screen(monitor_index: i8) -> Result<(DynamicImage, u32, u32), String> {
    let monitors = Monitor::all().map_err(|e| format!("Erreur Monitor::all: {}", e))?;

    let monitor = monitors
        .into_iter()
        .nth(monitor_index as usize)
        .ok_or(format!("Écran {} non trouvé", monitor_index))?;

    let image = monitor
        .capture_image()
        .map_err(|e| format!("Erreur capture écran: {}", e))?;

    let width = image.width();
    let height = image.height();

    let rgba =
        RgbaImage::from_raw(width, height, image.into_raw()).ok_or("Buffer RGBA invalide")?;

    Ok((DynamicImage::ImageRgba8(rgba), width, height))
}

pub fn crop_image_crop_position(image: DynamicImage, config: CropPosition) -> DynamicImage {
    let w = image.width();
    let h = image.height();
    let pixel = config.to_pixels(w, h);
    image.crop_imm(pixel.0, pixel.1, pixel.2, pixel.3)
}
