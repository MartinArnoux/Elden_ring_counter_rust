// ocr.rs - Version optimisée pour détection de mort uniquement

use crate::structs::settings::crop_position::CropPosition;
use crate::utils::image_processing::{
    extract_red_channel, has_red_text_present, preprocess_v1_fast, preprocess_v2_fallback,
    process_boss_gamma, process_boss_gamma_contrast,
};
use crate::utils::screen_capture::crop_image_crop_position;
use image::DynamicImage;
use rayon::prelude::*;
use std::collections::HashMap;
use strsim::jaro_winkler;
use uni_ocr::{OcrEngine, OcrProvider};

// ============================================================================
// DÉTECTION DE MORT (avec pré-filtre)
// ============================================================================

pub async fn detect_death(
    full_screen: &DynamicImage,
    death_zone_config: &CropPosition,
    death_text: String,
) -> Result<bool, String> {
    let _t0 = std::time::Instant::now();

    // ───────────────── Crop de la zone de mort
    let _t = std::time::Instant::now();
    let death_zone = death_zone_config.crop_image(full_screen);
    lap!(_t, "Crop zone");

    // ───────────────── Save debug crop
    #[cfg(feature = "debug")]
    {
        let _t = std::time::Instant::now();
        death_zone.save("crop_dead_zone.png").ok();
        lap!(_t, "Save crop (disk)");
    }

    // ───────────────── Pré-filtre rouge
    let _t = std::time::Instant::now();
    if !has_red_text_present(&death_zone) {
        lap!(_t, "Pré-filtre rouge (FAIL)");
        lap!(_t0, "TOTAL detect_death");
        return Ok(false);
    }
    lap!(_t, "Pré-filtre rouge (OK)");

    // ───────────────── Save écran complet
    #[cfg(feature = "debug")]
    {
        let _t = std::time::Instant::now();
        full_screen.save("all_screen_at_death.png").ok();
        lap!(_t, "Save screen (disk)");
    }

    // ───────────────── Préprocess OCR
    let _t = std::time::Instant::now();
    lap!(_t, "Preprocess OCR");

    // ───────────────── Engine OCR
    let _t = std::time::Instant::now();
    let engine = OcrEngine::new(OcrProvider::Auto).map_err(|e| format!("OCR Engine: {}", e))?;
    lap!(_t, "Init OCR engine");

    // ───────────────── Preprocess + OCR V1
    let red = extract_red_channel(&death_zone);
    let v1 = preprocess_v1_fast(&red);
    let (ok_v1, score_v1) = ocr_check(&engine, &v1, death_text.clone(), "OCR version 1").await;
    if ok_v1 {
        lap!(_t0, "TOTAL detect_death");
        return Ok(true);
    }

    let v2 = preprocess_v2_fallback(&red);
    let (ok_v2, score_v2) = ocr_check(&engine, &v2, death_text.clone(), "OCR version 2").await;
    if ok_v2 {
        lap!(_t0, "TOTAL detect_death");
        return Ok(true);
    }
    lap!(
        _t0,
        format!(
            "Death detected : score 1 : {} ; score 2 : {}",
            score_v1, score_v2
        )
    );

    if score_v1 > 80. || score_v2 > 80. {
        #[cfg(feature = "debug")]
        {
            lap!(_t0, "TOTAL detect_death");
            println!("OCR V1 score: {}", score_v1);
            println!("OCR V2 score: {}", score_v2);
        }
        #[cfg(feature = "timing")]
        {
            v1.save("death_v1.png");
            v2.save("death_v2.png");
        }

        lap!(
            _t0,
            format!("Death detected : score 1 {} ; score {}", score_v1, score_v2)
        );
        return Ok(true);
    }

    #[cfg(feature = "debug")]
    {
        println!("rouge mais pas death");
        death_zone.save("death_zone.png");
        v1.save("death_v1.png");
        v2.save("death_v2.png");
    }
    Ok(false)
}

async fn ocr_check(
    engine: &OcrEngine,
    image: &DynamicImage,
    death_text: String,
    _label: &str,
) -> (bool, f64) {
    let _t = std::time::Instant::now();

    let detected = match engine.recognize_image(image).await {
        Ok((text, _, _)) => is_death_text(&text, death_text),
        Err(_) => (false, 0.0),
    };
    lap!(_t, _label);
    detected
}
fn is_death_text(text: &str, death_text: String) -> (bool, f64) {
    let upper = text.to_uppercase();
    let normalized = upper
        .replace("Ü", "U")
        .replace("È", "E")
        .replace("É", "E")
        .replace(" ", "");
    let cleaned2 = clean_ocr_text_universal(&normalized);
    let death_text_str = death_text.as_str();
    let death_texte_no_space = death_text_str.replace(" ", "");
    #[cfg(feature = "timing")]
    {
        println!("Death texte : {}", death_text_str);
        println!("Normalized text: {}", normalized);
        println!("Cleaned text: {}", cleaned2);
    }
    let similarity = jaro_winkler(&cleaned2, death_text_str) * 100.;

    let is_death =
        upper.contains(death_text_str) || cleaned2.contains(death_texte_no_space.as_str());
    (is_death, similarity)
}

// ============================================================================
// DÉTECTION DES BOSS (appelé seulement après détection de mort)
// ============================================================================

pub async fn get_boss_names(
    full_screen: DynamicImage,
    boss_zones: Vec<CropPosition>,
) -> Result<Vec<String>, String> {
    let _t = std::time::Instant::now();
    #[cfg(feature = "debug")]
    {
        println!("Début de la recherche des noms des boss");
    }

    let mut bosses = Vec::new();

    #[cfg(feature = "debug")]
    let mut debug_images = Vec::new();

    // Traiter chaque zone séquentiellement
    for (zone_index, zone) in boss_zones.iter().enumerate() {
        let _t1 = std::time::Instant::now();
        let _t0 = std::time::Instant::now();
        let cropped = crop_image_crop_position(full_screen.clone(), *zone);
        lap!(_t0, format!("Crop zone {}", zone_index + 1));
        #[cfg(feature = "debug")]
        debug_images.push(cropped.clone());
        let _t0 = std::time::Instant::now();
        let candidates = get_boss_name(cropped.clone()).await?;
        lap!(_t0, format!("Get Boss Name for boss {}", zone_index + 1));
        // Vérifier le meilleur candidat
        let Some((best_text, best_score)) = candidates.first() else {
            println!("⚠️ Zone {} : Aucun candidat trouvé", zone_index + 1);
            break; // Si pas de résultat, arrêter la recherche
        };

        println!(
            "✅ Zone {} - Meilleur résultat (score {:.2}): {}",
            zone_index + 1,
            best_score,
            best_text
        );

        // Vérifier le score minimum
        if *best_score <= 5.0 {
            println!(
                "⚠️ Zone {} : Score trop faible, arrêt de la recherche",
                zone_index + 1
            );
            break; // Score insuffisant, arrêter
        }

        // Ajouter le boss trouvé
        bosses.push(best_text.clone());

        lap!(_t1, format!("Temps traitement zone {}", zone_index + 1));
        #[cfg(feature = "timing")]
        {
            let _ = cropped
                .save(format!("boss_zone_{}.png", zone_index + 1))
                .ok();
        }
    }

    // Debug: sauvegarder toutes les images
    #[cfg(feature = "debug")]
    {
        println!("Saving all images");
        if full_screen.width() > 0 && full_screen.height() > 0 {
            full_screen.save("all_image.png").ok();
        } else {
            eprintln!("⚠️ full_screen est vide");
        }

        for (i, img) in debug_images.iter().enumerate() {
            if img.width() > 0 && img.height() > 0 {
                println!("Saving boss_zone_{}.png", i + 1);
                img.save(format!("boss_zone_{}.png", i + 1)).ok();
            } else {
                eprintln!("⚠️ boss_zone_{} est vide", i + 1);
            }
        }
    }
    lap!(_t, "Temps Total pour les noms des boss  :  ");
    Ok(bosses)
}

pub async fn get_boss_name(
    dyn_image: DynamicImage,
) -> Result<Vec<(std::string::String, f64)>, String> {
    // Tester plusieurs prétraitements
    let _t0 = std::time::Instant::now();

    let params = vec![
        (0.20, false, 0.0),
        (0.25, false, 0.0),
        (0.30, false, 0.0),
        (0.35, false, 0.0),
        (0.40, false, 0.0),
        (0.30, true, 1.3),
        (0.30, true, 1.5),
    ];

    // Traitement parallèle
    let versions: Vec<_> = params
        .par_iter() // rayon parallel iterator
        .map(|(gamma, contrast, c)| {
            if *contrast {
                process_boss_gamma_contrast(&dyn_image, *gamma, *c)
            } else {
                process_boss_gamma(&dyn_image, *gamma)
            }
        })
        .collect();

    lap!(_t0, "Fin process 1 boss");
    let _t1 = std::time::Instant::now();
    let mut candidates: Vec<(String, f64)> = Vec::new();
    let engine =
        OcrEngine::new(OcrProvider::Auto).map_err(|e| format!("Erreur OCR Engine: {}", e))?;
    for (idx, version) in versions.iter().enumerate() {
        if let Ok((text, _, _)) = engine.recognize_image(&version).await {
            let cleaned = clean_ocr_text_universal(&text);
            if !cleaned.is_empty() {
                let score = calculate_universal_text_quality(&cleaned, &text);
                candidates.push((cleaned.clone(), score));
                println!("Version {}: '{}' (score: {:.2})", idx, cleaned, score);
            }
        }
        #[cfg(feature = "debug")]
        {
            use std::time::{SystemTime, UNIX_EPOCH};

            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(); // millisecondes depuis 1970
            let filename = format!("boss_version_{}_{}.png", idx, ts);
            version.save(&filename).unwrap();
        }
    }
    lap!(_t1, "Fin calcule ocr + score 1 bosse");
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
    Ok(candidates.clone())
}

// Nettoyage universel pour toutes les langues
fn clean_ocr_text_universal(text: &str) -> String {
    let cleaned = text
        .trim()
        .chars()
        .filter(|c| {
            // Garder :
            // - Lettres de toutes langues (Unicode)
            // - Espaces
            // - Apostrophes, tirets
            // - Caractères de ponctuation commune dans les noms
            c.is_alphabetic() || c.is_whitespace() || *c == '\'' || *c == '-' || *c == ','
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    cleaned
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
    use crate::structs::settings::game::Game;
    use crate::structs::settings::settings::Settings;

    #[tokio::test]
    async fn test_boss_detection_eldenring() {
        let img = image::open("all_image.png").unwrap();
        let mut settings = Settings::default();
        settings.set_game(Game::EldenRing);
        println!("Zone : {:?}", settings.get_game_config().get_boss_zones());

        let boss_zones = settings.get_game_config().get_boss_zones().clone();
        let bosses = get_boss_names(img, boss_zones).await.unwrap();

        println!("Bosses détectés : {:?}", bosses);
        assert!(!bosses.is_empty(), "Aucun boss détecté !");
    }
    #[tokio::test]
    async fn test_boss_detection_eldenring_with_boss_zones() {
        let img = image::open("boss_zone_1.png").unwrap();
        let mut settings = Settings::default();
        settings.set_game(Game::EldenRing);
        let bosses = get_boss_name(img).await.unwrap();

        println!("Bosses détectés : {:?}", bosses);
        assert!(!bosses.is_empty(), "Aucun boss détecté !");
    }
}
