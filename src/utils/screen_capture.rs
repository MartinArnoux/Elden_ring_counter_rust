// src/utils/screen_capture.rs
use image::DynamicImage;
use image::RgbaImage;
use xcap::Monitor;
macro_rules! lap {
    ($start:expr, $label:expr) => {{
        #[cfg(feature = "timing")]
        {
            let elapsed = $start.elapsed();
            println!("{:<30} {:>6} ms", $label, elapsed.as_millis());
        }
    }};
}
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
