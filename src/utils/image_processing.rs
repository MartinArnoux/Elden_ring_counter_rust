use image::{DynamicImage, GrayImage, ImageBuffer, Luma};

//////////////////////////////////////////////////////////////////
/////////////////////PROCESSING DEATH/////////////////////////////
//////////////////////////////////////////////////////////////////
pub fn adjust_gamma(img: &GrayImage, gamma: f32) -> GrayImage {
    let mut result = img.clone();
    for pixel in result.pixels_mut() {
        let normalized = pixel[0] as f32 / 255.0;
        let corrected = normalized.powf(gamma);
        pixel[0] = (corrected * 255.0) as u8;
    }
    result
}
pub fn increase_contrast(img: &GrayImage, factor: f32) -> GrayImage {
    let mut result = img.clone();
    for pixel in result.pixels_mut() {
        let value = pixel[0] as f32;
        let new_value = ((value - 128.0) * factor + 128.0).clamp(0.0, 255.0);
        pixel[0] = new_value as u8;
    }
    result
}
// Fonction pour détecter la présence de "texte" rouge dans une image
pub fn has_red_text_present(image: &DynamicImage) -> bool {
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

pub fn extract_red_channel(image: &DynamicImage) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let rgba = image.to_rgba8();
    ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
        let pixel = rgba.get_pixel(x, y);
        Luma([pixel[0]])
    })
}
pub fn preprocess_v1_fast(red: &ImageBuffer<Luma<u8>, Vec<u8>>) -> DynamicImage {
    use std::time::Instant;

    let _t = Instant::now();
    let _t0 = Instant::now();
    let brightened = adjust_gamma(red, 0.4);
    lap!(_t, "PREPROCESS |   gamma 0.4          {:>6} ms");
    let (w, h) = brightened.dimensions();

    const TARGET_HEIGHT: u32 = 120;
    let scale = TARGET_HEIGHT as f32 / h as f32;

    let _t0 = Instant::now();
    let scaled = DynamicImage::ImageLuma8(brightened).resize(
        (w as f32 * scale) as u32,
        TARGET_HEIGHT,
        image::imageops::FilterType::CatmullRom,
    );
    lap!(_t0, "PREPROCESS |   scaled         {:>6} ms");
    lap!(_t, "PREPROCESS |   total time");
    scaled
}

pub fn preprocess_v2_fallback(red: &ImageBuffer<Luma<u8>, Vec<u8>>) -> DynamicImage {
    use std::time::Instant;

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
//////////////////////////////////////////////////////////////////
/////////////////////PROCESSING BOSSES////////////////////////////
//////////////////////////////////////////////////////////////////

pub fn process_boss_gamma(dyn_image: &DynamicImage, gamma: f32) -> DynamicImage {
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

pub fn process_boss_gamma_contrast(
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
