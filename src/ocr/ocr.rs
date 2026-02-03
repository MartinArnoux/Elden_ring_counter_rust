// ocr.rs - Version optimisée pour détection de mort uniquement
macro_rules! lap {
    ($start:expr, $label:expr) => {{
        #[cfg(feature = "timing")]
        {
            let elapsed = $start.elapsed();
            println!("{:<30} {:>6} ms", $label, elapsed.as_millis());
        }
    }};
}
use image::{DynamicImage, GrayImage, ImageBuffer, Luma, RgbaImage};
use uni_ocr::{OcrEngine, OcrProvider};
use xcap::Monitor;
// ============================================================================
// FONCTIONS DE PRÉTRAITEMENT
// ============================================================================

fn adjust_gamma(img: &GrayImage, gamma: f32) -> GrayImage {
    let mut result = img.clone();
    for pixel in result.pixels_mut() {
        let normalized = pixel[0] as f32 / 255.0;
        let corrected = normalized.powf(gamma);
        pixel[0] = (corrected * 255.0) as u8;
    }
    result
}

fn increase_contrast(img: &GrayImage, factor: f32) -> GrayImage {
    let mut result = img.clone();
    for pixel in result.pixels_mut() {
        let value = pixel[0] as f32;
        let new_value = ((value - 128.0) * factor + 128.0).clamp(0.0, 255.0);
        pixel[0] = new_value as u8;
    }
    result
}

// ============================================================================
// CAPTURE D'ÉCRAN
// ============================================================================

fn capture_screen() -> Result<(DynamicImage, u32, u32), String> {
    // Récupérer tous les écrans
    let monitors = Monitor::all().map_err(|e| format!("Erreur Monitor::all: {}", e))?;

    let mut primary = None;

    for m in monitors {
        match m.is_primary() {
            Ok(true) => {
                primary = Some(m);
                break;
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("⚠️ Erreur is_primary: {}", e);
            }
        }
    }

    let monitor = primary.ok_or("Aucun écran principal détecté")?;
    // Capture
    let image = monitor
        .capture_image()
        .map_err(|e| format!("Erreur capture écran: {}", e))?;

    let width = image.width();
    let height = image.height();

    // xcap fournit déjà du RGBA
    let rgba =
        RgbaImage::from_raw(width, height, image.into_raw()).ok_or("Buffer RGBA invalide")?;

    Ok((DynamicImage::ImageRgba8(rgba), width, height))
}

// ============================================================================
// PRÉ-FILTRE RAPIDE : DÉTECTION DE ROUGE
// ============================================================================

fn has_red_text_present(image: &DynamicImage) -> bool {
    let rgba = image.to_rgba8();
    let mut red_pixel_count = 0;
    let total_pixels = (image.width() * image.height()) as usize;
    for p in rgba.pixels() {
        let r = p[0] as i32;
        let g = p[1] as i32;
        let b = p[2] as i32;

        // Rouge dominant sombre (Elden Ring)
        if r > 60 && r > g + 20 && r > b + 20 {
            red_pixel_count += 1;
        }
    }

    let pct = red_pixel_count as f32 / total_pixels as f32;

    pct > 0.01 // 1% suffit largement
}

// ============================================================================
// PRÉTRAITEMENT OPTIMISÉ (2 versions seulement)
// ============================================================================

fn extract_red_channel(image: &DynamicImage) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let rgba = image.to_rgba8();
    ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
        let pixel = rgba.get_pixel(x, y);
        Luma([pixel[0]])
    })
}

fn preprocess_v1_fast(red: &ImageBuffer<Luma<u8>, Vec<u8>>) -> DynamicImage {
    use std::time::Instant;

    let _t = Instant::now();
    let brightened = adjust_gamma(red, 0.4);
    lap!(_t, "PREPROCESS |   gamma 0.4          {:>6} ms");
    let (w, h) = brightened.dimensions();

    const TARGET_HEIGHT: u32 = 120;
    let scale = TARGET_HEIGHT as f32 / h as f32;

    let _t = Instant::now();
    let scaled = DynamicImage::ImageLuma8(brightened).resize(
        (w as f32 * scale) as u32,
        TARGET_HEIGHT,
        image::imageops::FilterType::CatmullRom,
    );
    lap!(_t, "PREPROCESS |   total time         {:>6} ms");
    scaled
}

fn preprocess_v2_fallback(red: &ImageBuffer<Luma<u8>, Vec<u8>>) -> DynamicImage {
    use std::time::Instant;

    lap!(t, "PREPROCESS | ── Version 2 (FALLBACK)");

    let _t = Instant::now();
    let gamma = adjust_gamma(red, 0.3);
    lap!(_t, "PREPROCESS |   gamma 0.3          {:>6} ms");

    let _t = Instant::now();
    let contrast = increase_contrast(&gamma, 2.0);
    lap!(_t, "PREPROCESS |   contrast x2.0     {:>6} ms");
    let (w, h) = contrast.dimensions();

    const TARGET_HEIGHT: u32 = 120;
    let scale = TARGET_HEIGHT as f32 / h as f32;

    let _t = Instant::now();
    let scaled = DynamicImage::ImageLuma8(contrast).resize(
        (w as f32 * scale) as u32,
        TARGET_HEIGHT,
        image::imageops::FilterType::CatmullRom,
    );

    lap!(_t, "PREPROCESS |   resize → 120px     {:>6} ms");

    scaled
}
/* fn preprocess_death_text_optimized(image: &DynamicImage) -> (DynamicImage, DynamicImage) {
    use std::time::Instant;

    let _t0 = Instant::now();

    let red = extract_red_channel(image);

    lap!(t, "PREPROCESS |   extract red channel {:>6} ms");
    let v1 = preprocess_v1_fast(&red);
    let v2 = preprocess_v2_fallback(&red);

    lap!(_t0, "════════ PREPROCESS TOTAL        {:>6} ms ════════");

    (v1, v2)
}*/

// ============================================================================
// DÉTECTION DE MORT (avec pré-filtre)
// ============================================================================

pub async fn detect_death() -> Result<Option<DynamicImage>, String> {
    let _t0 = std::time::Instant::now();

    // ───────────────── Capture écran
    let _t = std::time::Instant::now();
    let (dyn_image, w, h) = tokio::task::spawn_blocking(|| capture_screen())
        .await
        .map_err(|e| format!("Erreur join: {}", e))?
        .map_err(|e| format!("Erreur capture: {}", e))?;
    lap!(_t, "Capture écran");

    // ───────────────── Crop
    let _t = std::time::Instant::now();
    let crop_x = (w * 33) / 100;
    let crop_y = (h * 46) / 100;
    let crop_width = (w * 35) / 100;
    let crop_height = (h * 10) / 100;
    let dead_zone = dyn_image.crop_imm(crop_x, crop_y, crop_width, crop_height);
    lap!(_t, "Crop zone");

    // // ───────────────── Save debug crop
    // let t = std::time::Instant::now();
    // dead_zone.save("crop_dead_zone.png").ok();
    // lap!(t, "Save crop (disk)");

    // ───────────────── Pré-filtre rouge
    let _t = std::time::Instant::now();
    if !has_red_text_present(&dead_zone) {
        lap!(t, "Pré-filtre rouge (FAIL)");
        lap!(t0, "TOTAL detect_death");
        return Ok(None);
    }
    lap!(_t, "Pré-filtre rouge (OK)");

    // ───────────────── Save écran complet
    // let t = std::time::Instant::now();
    // dyn_image.save("all_screen_at_death.png").ok();
    // lap!(t, "Save screen (disk)");

    // ───────────────── Préprocess OCR
    let _t = std::time::Instant::now();
    //let versions = preprocess_death_text_optimized(&dead_zone);
    lap!(_t, "Preprocess OCR");

    // ───────────────── Engine OCR
    let _t = std::time::Instant::now();
    let engine = OcrEngine::new(OcrProvider::Auto).map_err(|e| format!("OCR Engine: {}", e))?;
    lap!(_t, "Init OCR engine");

    // ───────────────── Preprocess + OCR V1
    let red = extract_red_channel(&dead_zone);

    let v1 = preprocess_v1_fast(&red);
    if ocr_check(&engine, &v1, "OCR version 1").await {
        lap!(_t0, "TOTAL detect_death");
        return Ok(Some(dyn_image));
    }

    let v2 = preprocess_v2_fallback(&red);
    if ocr_check(&engine, &v2, "OCR version 2").await {
        lap!(_t0, "TOTAL detect_death");
        return Ok(Some(dyn_image));
    }

    return Ok(None);
}

async fn ocr_check(engine: &OcrEngine, image: &DynamicImage, _label: &str) -> bool {
    let _t = std::time::Instant::now();

    let detected = match engine.recognize_image(image).await {
        Ok((text, _, _)) => is_death_text(&text),
        Err(_) => false,
    };

    lap!(_t, _label);
    detected
}
fn is_death_text(text: &str) -> bool {
    let upper = text.to_uppercase();
    let normalized = upper
        .replace("Ü", "U")
        .replace("È", "E")
        .replace("É", "E")
        .replace(" ", "");

    upper.contains("YOU DIED")
        || upper.contains("VOUS AVEZ PERI")
        || normalized.contains("VOUSAVEZPERI")
}

// ============================================================================
// DÉTECTION DES BOSS (appelé seulement après détection de mort)
// ============================================================================

pub async fn get_boss_names(dyn_image: DynamicImage) -> Result<Vec<String>, String> {
    let w = dyn_image.width();
    let h = dyn_image.height();
    let mut bosses = Vec::new();

    let crop_x = (w * 24) / 100;
    let crop_y = (h * 77) / 100;
    let crop_width = (w * 53) / 100;
    let crop_height = (h * 5) / 100;

    let engine =
        OcrEngine::new(OcrProvider::Auto).map_err(|e| format!("Erreur OCR Engine: {}", e))?;

    // Boss principal
    let boss_zone_1 = dyn_image.crop_imm(crop_x, crop_y, crop_width, crop_height);

    // Essayer plusieurs prétraitements
    let versions = vec![
        ("adaptive", process_boss_adaptive(&boss_zone_1)),
        ("contrast_1.5", process_boss_contrast(&boss_zone_1, 1.5)),
        ("contrast_2.0", process_boss_contrast(&boss_zone_1, 2.0)),
    ];

    let mut best_text = String::new();
    let mut best_length = 0; // On utilise la longueur comme heuristique

    for (name, version) in versions {
        if let Ok((text, _, _)) = engine.recognize_image(&version).await {
            let cleaned = text.trim().to_string();
            // On garde le texte le plus long et qui semble valide
            if !cleaned.is_empty() && cleaned.len() > best_length && cleaned.len() > 5 {
                best_text = cleaned;
                best_length = best_text.len();
                println!("Meilleur résultat avec {}: {}", name, best_text);
            }
        }
    }

    if !best_text.is_empty() {
        bosses.push(best_text);

        // Boss secondaire
        let crop_y_2 = crop_y.saturating_sub((h * 5) / 100);
        let boss_zone_2 = dyn_image.crop_imm(crop_x, crop_y_2, crop_width, crop_height);

        // Essayer les mêmes prétraitements pour le boss secondaire
        let versions2 = vec![
            process_boss_adaptive(&boss_zone_2),
            process_boss_contrast(&boss_zone_2, 1.5),
            process_boss_contrast(&boss_zone_2, 2.0),
        ];

        let mut best_text2 = String::new();
        let mut best_length2 = 0;

        for version in versions2 {
            if let Ok((text2, _, _)) = engine.recognize_image(&version).await {
                let cleaned = text2.trim().to_string();
                if !cleaned.is_empty() && cleaned.len() > best_length2 && cleaned.len() > 5 {
                    best_text2 = cleaned;
                    best_length2 = best_text2.len();
                }
            }
        }

        if !best_text2.is_empty() {
            bosses.push(best_text2);
        }
    }

    Ok(bosses)
}

fn process_boss_adaptive(dyn_image: &DynamicImage) -> DynamicImage {
    let gray = dyn_image.to_luma8();
    let binary = adaptive_threshold(&gray, 25);

    let (w, h) = binary.dimensions();
    DynamicImage::ImageLuma8(binary).resize(w * 3, h * 3, image::imageops::FilterType::Lanczos3)
}

fn process_boss_contrast(dyn_image: &DynamicImage, factor: f32) -> DynamicImage {
    let gray = dyn_image.to_luma8();
    let contrasted = increase_contrast(&gray, factor);

    let (w, h) = contrasted.dimensions();
    DynamicImage::ImageLuma8(contrasted).resize(w * 3, h * 3, image::imageops::FilterType::Lanczos3)
}

fn adaptive_threshold(img: &GrayImage, block_size: u32) -> GrayImage {
    let (width, height) = img.dimensions();
    let mut result = GrayImage::new(width, height);
    let half_block = block_size / 2;

    for y in 0..height {
        for x in 0..width {
            let y_start = y.saturating_sub(half_block);
            let y_end = (y + half_block + 1).min(height);
            let x_start = x.saturating_sub(half_block);
            let x_end = (x + half_block + 1).min(width);

            let mut sum = 0u32;
            let mut count = 0u32;

            for ly in y_start..y_end {
                for lx in x_start..x_end {
                    sum += img.get_pixel(lx, ly)[0] as u32;
                    count += 1;
                }
            }

            let local_mean = (sum / count) as u8;
            let pixel_value = img.get_pixel(x, y)[0];

            // Si le pixel est plus sombre que la moyenne locale - 10, c'est du texte
            result.put_pixel(
                x,
                y,
                Luma([if pixel_value < local_mean.saturating_sub(10) {
                    0
                } else {
                    255
                }]),
            );
        }
    }

    result
}
