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
use std::collections::HashMap;

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

fn capture_screen(monitor_index: usize) -> Result<(DynamicImage, u32, u32), String> {
    let monitors = Monitor::all().map_err(|e| format!("Erreur Monitor::all: {}", e))?;

    let monitor = monitors
        .into_iter()
        .nth(monitor_index)
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
    let (dyn_image, w, h) = tokio::task::spawn_blocking(|| capture_screen(1))
        .await
        .map_err(|e| format!("Erreur join: {}", e))?
        .map_err(|e| format!("Erreur capture: {}", e))?;
    lap!(_t, "Capture écran");

    // ───────────────── Crop
    let _t = std::time::Instant::now();
    let crop_x = (w * 31) / 100;
    let crop_y = (h * 46) / 100;
    let crop_width = (w * 39) / 100;
    let crop_height = (h * 10) / 100;
    let dead_zone = dyn_image.crop_imm(crop_x, crop_y, crop_width, crop_height);
    lap!(_t, "Crop zone");

    // ───────────────── Save debug crop
    #[cfg(feature = "debug")]
    {
        let _t = std::time::Instant::now();
        dead_zone.save("crop_dead_zone.png").ok();
        lap!(_t, "Save crop (disk)");
    }
    // ───────────────── Pré-filtre rouge
    let _t = std::time::Instant::now();
    if !has_red_text_present(&dead_zone) {
        lap!(t, "Pré-filtre rouge (FAIL)");
        lap!(t0, "TOTAL detect_death");
        return Ok(None);
    }
    lap!(_t, "Pré-filtre rouge (OK)");

    // ───────────────── Save écran complet
    #[cfg(feature = "debug")]
    {
        let t = std::time::Instant::now();
        dyn_image.save("all_screen_at_death.png").ok();
        lap!(t, "Save screen (disk)");
    }
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
    println!("Début de la recherche des noms des boss");
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

    // Tester plusieurs prétraitements
    let versions = vec![
        process_boss_gamma(&boss_zone_1, 0.20),
        process_boss_gamma(&boss_zone_1, 0.25),
        process_boss_gamma(&boss_zone_1, 0.30),
        process_boss_gamma(&boss_zone_1, 0.35),
        process_boss_gamma(&boss_zone_1, 0.40),
        process_boss_gamma_contrast(&boss_zone_1, 0.30, 1.3),
        process_boss_gamma_contrast(&boss_zone_1, 0.30, 1.5),
    ];

    let mut candidates: Vec<(String, f64)> = Vec::new();

    for (idx, version) in versions.iter().enumerate() {
        if let Ok((text, _, _)) = engine.recognize_image(&version).await {
            let cleaned = clean_ocr_text_universal(&text);
            if !cleaned.is_empty() {
                let score = calculate_universal_text_quality(&cleaned, &text);
                candidates.push((cleaned.clone(), score));
                println!("Version {}: '{}' (score: {:.2})", idx, cleaned, score);
            }
        }
    }

    let mut freq = HashMap::new();
    for (text, _) in &candidates {
        *freq.entry(text.clone()).or_insert(0usize) += 1;
    }

    for (text, score) in candidates.iter_mut() {
        if let Some(count) = freq.get(text) {
            *score += (*count as f64) * 2.5;
        }
    }
    // Trier par score décroissant
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    if let Some((best_text, best_score)) = candidates.first() {
        println!(
            "✅ Meilleur résultat (score {:.2}): {}",
            best_score, best_text
        );

        if *best_score > 5.0 {
            bosses.push(best_text.clone());

            // Boss secondaire
            let crop_y_2 = crop_y.saturating_sub((h * 5) / 100);
            let boss_zone_2 = dyn_image.crop_imm(crop_x, crop_y_2, crop_width, crop_height);

            let versions2 = vec![
                process_boss_gamma(&boss_zone_2, 0.20),
                process_boss_gamma(&boss_zone_2, 0.25),
                process_boss_gamma(&boss_zone_2, 0.30),
                process_boss_gamma(&boss_zone_2, 0.35),
            ];

            let mut best_text2 = String::new();
            let mut best_score2 = 0.0;

            for version in versions2.clone() {
                if let Ok((text2, _, _)) = engine.recognize_image(&version).await {
                    let cleaned2 = clean_ocr_text_universal(&text2);
                    let score2 = calculate_universal_text_quality(&cleaned2, &text2);
                    if score2 > best_score2 {
                        best_score2 = score2;
                        best_text2 = cleaned2;
                    }
                }
            }

            if !best_text2.is_empty() && best_score2 > 5.0 {
                bosses.push(best_text2);
            }
            #[cfg(feature = "debug")]
            {
                dyn_image.save("all_image.png").unwrap();
                boss_zone_1.save("boss_zone_1.png").unwrap();
                boss_zone_2.save("boss_zone_2.png").unwrap();
                let mut i = 0;
                for version in versions {
                    version.save(format!("version_{}.png", i)).unwrap();
                    i += 1;
                }
                let mut i = 0;
                for version in versions2 {
                    version.save(format!("version_2_{}.png", i)).unwrap();
                    i += 1;
                }
            }
        }
    }

    Ok(bosses)
}

// Nettoyage universel pour toutes les langues
fn clean_ocr_text_universal(text: &str) -> String {
    text.trim()
        .chars()
        .filter(|c| {
            // Garder :
            // - Lettres de toutes langues (Unicode)
            // - Espaces
            // - Apostrophes, tirets
            // - Caractères de ponctuation commune dans les noms
            c.is_alphabetic()
                || c.is_whitespace()
                || *c == '\''
                || *c == '-'
                || *c == ','
                || *c == ':'
                || *c == '.'
                || *c == '('
                || *c == ')'
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// Score de qualité universel (pas de langue spécifique)
fn calculate_universal_text_quality(cleaned: &str, original: &str) -> f64 {
    let mut score = 0.0;

    // 1. LONGUEUR : Les noms de boss ont généralement 5-60 caractères
    let len = cleaned.len();
    if len >= 8 && len <= 50 {
        score += 15.0; // Longueur optimale
    } else if len >= 5 && len <= 70 {
        score += 8.0; // Acceptable
    } else if len < 5 {
        return 0.0; // Trop court
    } else {
        score -= 5.0; // Trop long
    }

    // 2. NOMBRE DE MOTS : généralement 1-6 mots
    let word_count = cleaned.split_whitespace().count();
    if word_count >= 2 && word_count <= 5 {
        score += 10.0; // Optimal
    } else if word_count == 1 {
        score += 5.0; // Un seul mot acceptable
    } else if word_count > 6 {
        score -= 5.0; // Trop de mots suspects
    }

    // 3. RATIO LETTRES vs TOTAL (doit être élevé)
    let letter_count = cleaned.chars().filter(|c| c.is_alphabetic()).count();
    let letter_ratio = letter_count as f64 / cleaned.len() as f64;
    score += letter_ratio * 20.0; // Max 20 points

    // 4. PÉNALITÉS pour caractères vraiment suspects (jamais dans des noms)
    let highly_suspicious = [
        '&', '@', '#', '$', '%', '*', '=', '+', '<', '>', '|', '\\', '/', '^', '~', '`', '{', '}',
        '[', ']', ';',
    ];
    let suspicious_count = cleaned
        .chars()
        .filter(|c| highly_suspicious.contains(c))
        .count();
    score -= suspicious_count as f64 * 8.0;

    // 5. PÉNALITÉ pour trop de chiffres (rare dans les noms)
    let digit_count = cleaned.chars().filter(|c| c.is_numeric()).count();
    if digit_count > 2 {
        score -= digit_count as f64 * 3.0;
    }

    // 6. BONUS si commence par une majuscule (universel)
    if cleaned.chars().next().map_or(false, |c| c.is_uppercase()) {
        score += 5.0;
    }

    // 7. PÉNALITÉ si beaucoup de différence entre original et nettoyé
    // (indique beaucoup de caractères invalides supprimés)
    let cleaning_diff = original.len().abs_diff(cleaned.len());
    if cleaning_diff > 15 {
        score -= (cleaning_diff as f64) * 0.3;
    }

    // 8. DÉTECTION de patterns répétitifs (artefacts OCR)
    if has_repetitive_patterns(cleaned) {
        score -= 12.0;
    }

    // 9. BONUS pour diversité des caractères (pas juste "aaaaa")
    let unique_chars = cleaned
        .chars()
        .filter(|c| c.is_alphabetic())
        .collect::<std::collections::HashSet<_>>()
        .len();
    let total_letters = cleaned.chars().filter(|c| c.is_alphabetic()).count();
    if total_letters > 0 {
        let diversity = unique_chars as f64 / total_letters as f64;
        if diversity > 0.4 && !has_consecutive_duplicates(cleaned) {
            score += 5.0; // Bonne diversité
        }
    }

    // 10. PÉNALITÉ pour sequences de ponctuation multiple
    if cleaned.contains("..") || cleaned.contains("--") || cleaned.contains(",,") {
        score -= 5.0;
    }

    // 11. BONUS si contient des espaces (noms composés courants)
    if cleaned.contains(' ') {
        score += 3.0;
    }
    if has_suspicious_final_duplication(cleaned) {
        score -= 8.0; // pénalité FORTE
    }

    score.max(0.0)
}

// Détecte les patterns répétitifs suspects (universel)
fn has_repetitive_patterns(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 4 {
        return false;
    }

    // Vérifier répétitions de 3+ caractères identiques
    for i in 0..chars.len().saturating_sub(2) {
        if chars[i] == chars[i + 1] && chars[i] == chars[i + 2] {
            return true;
        }
    }

    // Vérifier patterns AB-AB-AB
    for i in 0..chars.len().saturating_sub(5) {
        if chars[i] == chars[i + 2] && chars[i] == chars[i + 4] && chars[i + 1] == chars[i + 3] {
            return true;
        }
    }

    false
}

fn process_boss_gamma(dyn_image: &DynamicImage, gamma: f32) -> DynamicImage {
    let gray = dyn_image.to_luma8();
    let mut enhanced = gray.clone();

    for pixel in enhanced.pixels_mut() {
        let val = pixel[0] as f32 / 255.0;
        let corrected = (val.powf(gamma) * 255.0).clamp(0.0, 255.0);
        pixel[0] = corrected as u8;
    }

    let (w, h) = enhanced.dimensions();
    DynamicImage::ImageLuma8(enhanced).resize(w * 4, h * 4, image::imageops::FilterType::Lanczos3)
}

fn process_boss_gamma_contrast(
    dyn_image: &DynamicImage,
    gamma: f32,
    contrast: f32,
) -> DynamicImage {
    let gray = dyn_image.to_luma8();
    let mut enhanced = gray.clone();

    for pixel in enhanced.pixels_mut() {
        let val = pixel[0] as f32 / 255.0;
        let gamma_corrected = val.powf(gamma);
        let contrasted = ((gamma_corrected - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
        pixel[0] = (contrasted * 255.0) as u8;
    }

    let (w, h) = enhanced.dimensions();
    DynamicImage::ImageLuma8(enhanced).resize(w * 4, h * 4, image::imageops::FilterType::Lanczos3)
}

fn has_suspicious_final_duplication(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    if n < 3 {
        return false;
    }

    // Dernier caractère répété
    chars[n - 1] == chars[n - 2]
}

fn has_consecutive_duplicates(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i] == chars[i + 1] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_boss_detection() {
        let img = image::open("all_image.png").unwrap();
        let bosses = get_boss_names(img).await.unwrap();

        println!("Bosses détectés : {:?}", bosses);
        assert!(!bosses.is_empty(), "Aucun boss détecté !");
    }
}
