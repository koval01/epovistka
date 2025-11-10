use image::{Rgba, RgbaImage, ImageEncoder, ExtendedColorType};
use rusttype::{Font, Scale, point};
use rand::Rng;
use std::sync::Arc;
use tracing::info;
use chrono::prelude::*;

use crate::models::generate::{GenerateRequest, GenerateError};

const TEMPLATE_PATH: &str = "assets/template.png";
const SIGN_PATH: &str = "assets/sign.png";
const WATERMARK_PATH: &str = "assets/watermark.png";
const FONT_PATH: &str = "assets/font.ttf";

#[derive(Debug, Clone)]
struct FieldPosition {
    x: f32,
    y: f32
}

#[derive(Debug)]
pub struct ImageGenerator {
    template: Arc<RgbaImage>,
    sign: Arc<RgbaImage>,
    watermark: Arc<RgbaImage>,
    font: Arc<Font<'static>>,
    fields: std::collections::HashMap<&'static str, Vec<FieldPosition>>,
    sign_positions: Vec<FieldPosition>,
}

impl ImageGenerator {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let template_image = image::open(TEMPLATE_PATH)
            .map_err(|e| format!("Failed to open template image: {}", e))?;
        let template = template_image.to_rgba8();

        // Load sign image
        let sign_image = image::open(SIGN_PATH)
            .map_err(|e| format!("Failed to open sign image: {}", e))?;
        let sign = sign_image.to_rgba8();

        // Load watermark image
        let watermark_image = image::open(WATERMARK_PATH)
            .map_err(|e| format!("Failed to open watermark image: {}", e))?;
        let watermark = watermark_image.to_rgba8();

        // Load font
        let font_data = std::fs::read(FONT_PATH)
            .map_err(|e| format!("Failed to read font file: {}", e))?;
        let font = Font::try_from_vec(font_data)
            .ok_or("Failed to load font from data")?;

        let fields = std::collections::HashMap::from([
            ("name", vec![
                FieldPosition { x: 255.0, y: 22.0 },
                FieldPosition { x: 365.0, y: 975.0 },
            ]),
            ("address", vec![
                FieldPosition { x: 305.0, y: 70.0 },
            ]),
            ("number", vec![
                FieldPosition { x: 560.0, y: 135.0 },
                FieldPosition { x: 448.0, y: 350.0 },
            ]),
            ("issuer", vec![
                FieldPosition { x: 305.0, y: 250.0 },
                FieldPosition { x: 265.0, y: 1020.0 },
            ]),
            ("issuer_position", vec![
                FieldPosition { x: 180.0, y: 730.0 },
            ]),
            ("issuer_initials", vec![
                FieldPosition { x: 732.0, y: 730.0 },
            ]),
            ("year", vec![
                FieldPosition { x: 367.0, y: 352.0 },
                FieldPosition { x: 676.0, y: 453.0 },
                FieldPosition { x: 392.0, y: 843.0 },
                FieldPosition { x: 483.0, y: 1096.0 },
                FieldPosition { x: 836.0, y: 1096.0 },
            ]),
            ("time", vec![
                FieldPosition { x: 760.0, y: 457.0 },
            ]),
        ]);

        // Define positions for signatures (adjust these coordinates as needed)
        let sign_positions = vec![
            FieldPosition { x: 618.0, y: 716.0 },
        ];

        Ok(Self {
            template: Arc::new(template),
            sign: Arc::new(sign),
            watermark: Arc::new(watermark),
            font: Arc::new(font),
            fields,
            sign_positions,
        })
    }

    pub async fn generate_image(
        &self,
        request: &GenerateRequest,
    ) -> Result<Vec<u8>, GenerateError> {
        let mut rng = rand::rng();
        let number = rng.random_range(64*64..512*512);

        // Get current year and generate time
        let current_year = Local::now().year().to_string().chars().skip(2).collect::<String>();
        let time_str = self.generate_time();

        // Create a copy of the template to work with
        let mut image = self.template.as_ref().clone();

        // Draw text fields
        self.draw_all_text(&mut image, request, number, &current_year, &time_str)
            .map_err(|e| GenerateError::GenerationError(e.to_string()))?;

        // Draw signatures
        self.draw_all_signatures(&mut image)
            .map_err(|e| GenerateError::GenerationError(e.to_string()))?;

        // Draw watermarks with unpredictable placement and duplication
        self.draw_all_watermarks(&mut image)
            .map_err(|e| GenerateError::GenerationError(e.to_string()))?;

        // Convert to bytes using the new image library API
        let mut bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut bytes);

        encoder
            .write_image(
                &image,
                image.width(),
                image.height(),
                ExtendedColorType::from(image::ColorType::Rgba8),
            )
            .map_err(|e| GenerateError::GenerationError(format!("Failed to encode PNG: {}", e)))?;

        info!("Successfully generated image for: {}", request.name);
        Ok(bytes)
    }

    fn generate_time(&self) -> String {
        let mut rng = rand::rng();
        let hour = rng.random_range(8..19); // 08 to 18
        let minute = rng.random_range(0..12) * 5; // 00, 05, 10, ..., 55

        format!("{:02}:{:02}", hour, minute)
    }

    fn draw_all_watermarks(&self, image: &mut RgbaImage) -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = rand::rng();

        // Generate random number of watermarks (2-5 copies)
        let num_watermarks = rng.random_range(5..10);

        for _ in 0..num_watermarks {
            // Random scale (smaller for more subtle effect)
            let scale_factor = rng.random_range(0.4..1.0);

            // Random rotation angle (-45 to 45 degrees)
            let rotation_degrees = rng.random_range(-45.0..45.0);

            // Random opacity (10-30% for subtle but visible effect)
            let opacity = rng.random_range(0.08..0.2);

            // Random position - allow watermarks to extend beyond image bounds
            let x_offset = rng.random_range(-350.0..(image.width() as f32 + 350.0));
            let y_offset = rng.random_range(-800.0..(image.height() as f32 + 800.0));

            self.draw_watermark_at_position(
                image,
                x_offset,
                y_offset,
                scale_factor,
                rotation_degrees,
                opacity,
            )?;
        }

        Ok(())
    }

    fn draw_watermark_at_position(
        &self,
        image: &mut RgbaImage,
        x: f32,
        y: f32,
        scale_factor: f32,
        rotation_degrees: f32,
        opacity: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let watermark = self.watermark.as_ref();

        // Calculate new dimensions
        let new_width = (watermark.width() as f32 * scale_factor) as u32;
        let new_height = (watermark.height() as f32 * scale_factor) as u32;

        // Resize watermark
        let resized_watermark = image::imageops::resize(
            watermark,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );

        let start_x = x as i32;
        let start_y = y as i32;
        let radians = rotation_degrees.to_radians();
        let cos_angle = radians.cos();
        let sin_angle = radians.sin();

        // Calculate center of rotation
        let center_x = resized_watermark.width() as f32 / 2.0;
        let center_y = resized_watermark.height() as f32 / 2.0;

        // Draw the rotated watermark
        for target_x in 0..image.width() {
            for target_y in 0..image.height() {
                // Calculate relative coordinates
                let rel_x = target_x as i32 - start_x;
                let rel_y = target_y as i32 - start_y;

                // Apply inverse rotation to find source pixel
                let src_x_f = (rel_x as f32 - center_x) * cos_angle + (rel_y as f32 - center_y) * sin_angle + center_x;
                let src_y_f = -(rel_x as f32 - center_x) * sin_angle + (rel_y as f32 - center_y) * cos_angle + center_y;

                let src_x = src_x_f.round() as i32;
                let src_y = src_y_f.round() as i32;

                if src_x >= 0 && src_x < resized_watermark.width() as i32 &&
                    src_y >= 0 && src_y < resized_watermark.height() as i32 {

                    let pixel = resized_watermark.get_pixel(src_x as u32, src_y as u32);

                    // Skip fully transparent pixels
                    if pixel[3] == 0 {
                        continue;
                    }

                    let background_pixel = image.get_pixel(target_x, target_y);

                    // Apply additional opacity to the watermark
                    let mut adjusted_pixel = *pixel;
                    adjusted_pixel[3] = (pixel[3] as f32 * opacity) as u8;

                    let blended = self.blend_pixels(*background_pixel, adjusted_pixel);
                    image.put_pixel(target_x, target_y, blended);
                }
            }
        }

        Ok(())
    }

    fn draw_all_text(
        &self,
        image: &mut RgbaImage,
        request: &GenerateRequest,
        number: u32,
        current_year: &str,
        time_str: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let color = Rgba([0, 50, 150, 255]);
        let mut rng = rand::rng();

        // Draw name
        if let Some(positions) = self.fields.get("name") {
            for position in positions {
                let x = position.x + rng.random_range(-2.0..3.0);
                let y = position.y + rng.random_range(-2.0..2.0);
                self.draw_text_at_position(image, &request.name, x, y, Scale::uniform(rng.random_range(26.0..38.0)), color)?;
            }
        }

        // Draw address
        if let Some(positions) = self.fields.get("address") {
            for position in positions {
                let x = position.x + rng.random_range(-2.0..8.0);
                let y = position.y + rng.random_range(-1.0..6.0);
                self.draw_text_at_position(image, &request.address, x, y, Scale::uniform(rng.random_range(29.0..34.0)), color)?;
            }
        }

        // Draw issuer
        if let Some(positions) = self.fields.get("issuer") {
            for position in positions {
                let x = position.x + rng.random_range(-2.0..3.0);
                let y = position.y + rng.random_range(-2.0..2.0);
                self.draw_text_at_position(image, "Офісу Президента України", x, y, Scale::uniform(rng.random_range(26.0..38.0)), color)?;
            }
        }

        // Draw issuer position
        if let Some(positions) = self.fields.get("issuer_position") {
            for position in positions {
                let x = position.x + rng.random_range(-12.0..5.0);
                let y = position.y + rng.random_range(-4.0..4.0);
                self.draw_text_at_position(image, "Президент України", x, y, Scale::uniform(rng.random_range(30.0..36.0)), color)?;
            }
        }

        // Draw issuer initials
        if let Some(positions) = self.fields.get("issuer_initials") {
            for position in positions {
                let x = position.x + rng.random_range(-2.0..1.4);
                let y = position.y + rng.random_range(-4.0..4.0);
                self.draw_text_at_position(image, "Зеленський В. О.", x, y, Scale::uniform(rng.random_range(31.0..34.0)), color)?;
            }
        }

        // Draw number
        let number_str = number.to_string();
        if let Some(positions) = self.fields.get("number") {
            for position in positions {
                let x = position.x + rng.random_range(-2.0..5.0);
                let y = position.y + rng.random_range(-2.2..1.0);
                self.draw_text_at_position(image, &number_str, x, y, Scale::uniform(rng.random_range(36.0..48.0)), color)?;
            }
        }

        // Draw year (using current year)
        if let Some(positions) = self.fields.get("year") {
            for position in positions {
                let x = position.x + rng.random_range(-1.7..1.7);
                let y = position.y + rng.random_range(-1.2..1.2);
                self.draw_text_at_position(image, current_year, x, y, Scale::uniform(rng.random_range(32.0..37.0)), color)?;
            }
        }

        // Draw time (using generated time)
        if let Some(positions) = self.fields.get("time") {
            for position in positions {
                let x = position.x + rng.random_range(-2.5..8.0);
                let y = position.y + rng.random_range(-2.2..2.2);
                self.draw_text_at_position(image, time_str, x, y, Scale::uniform(rng.random_range(27.0..34.0)), color)?;
            }
        }

        Ok(())
    }

    fn draw_all_signatures(&self, image: &mut RgbaImage) -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = rand::rng();

        for position in &self.sign_positions {
            let scale_factor = rng.random_range(0.065..0.08); // Scale down the signature
            let x_offset = rng.random_range(-3.0..3.0);
            let y_offset = rng.random_range(-2.0..5.0);

            self.draw_signature_at_position(
                image,
                position.x + x_offset,
                position.y + y_offset,
                scale_factor
            )?;
        }

        Ok(())
    }

    fn draw_signature_at_position(
        &self,
        image: &mut RgbaImage,
        x: f32,
        y: f32,
        scale_factor: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sign = self.sign.as_ref();

        // Calculate new dimensions
        let new_width = (sign.width() as f32 * scale_factor) as u32;
        let new_height = (sign.height() as f32 * scale_factor) as u32;

        // Resize signature
        let resized_sign = image::imageops::resize(
            sign,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );

        let start_x = x as i32;
        let start_y = y as i32;

        // Draw the resized signature
        for (sx, sy, pixel) in resized_sign.enumerate_pixels() {
            let target_x = start_x + sx as i32;
            let target_y = start_y + sy as i32;

            if target_x >= 0 && target_x < image.width() as i32 &&
                target_y >= 0 && target_y < image.height() as i32 {

                // Blend the signature pixel with the background
                let background_pixel = image.get_pixel(target_x as u32, target_y as u32);
                let blended = self.blend_pixels(*background_pixel, *pixel);
                image.put_pixel(target_x as u32, target_y as u32, blended);
            }
        }

        Ok(())
    }

    fn blend_pixels(&self, background: Rgba<u8>, foreground: Rgba<u8>) -> Rgba<u8> {
        let alpha = foreground[3] as f32 / 255.0;

        let r = (foreground[0] as f32 * alpha + background[0] as f32 * (1.0 - alpha)) as u8;
        let g = (foreground[1] as f32 * alpha + background[1] as f32 * (1.0 - alpha)) as u8;
        let b = (foreground[2] as f32 * alpha + background[2] as f32 * (1.0 - alpha)) as u8;

        // Keep the alpha channel from background or use max of both
        let a = background[3].max(foreground[3]);

        Rgba([r, g, b, a])
    }

    fn draw_text_at_position(
        &self,
        image: &mut RgbaImage,
        text: &str,
        x: f32,
        y: f32,
        scale: Scale,
        color: Rgba<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let v_metrics = self.font.v_metrics(scale);
        let offset = point(x, y + v_metrics.ascent);

        let glyphs: Vec<_> = self.font.layout(text, scale, offset).collect();

        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, gv| {
                    let gx = gx as i32 + bounding_box.min.x;
                    let gy = gy as i32 + bounding_box.min.y;

                    if gx >= 0 && gx < image.width() as i32 && gy >= 0 && gy < image.height() as i32 {
                        let alpha = (gv * 255.0) as u8;
                        let pixel = image.get_pixel_mut(gx as u32, gy as u32);

                        // Blend the text color with the background
                        let blended = self.blend_colors(*pixel, color, alpha);
                        *pixel = blended;
                    }
                });
            }
        }

        Ok(())
    }

    fn blend_colors(&self, background: Rgba<u8>, foreground: Rgba<u8>, alpha: u8) -> Rgba<u8> {
        let alpha_f = alpha as f32 / 255.0;

        let r = (foreground[0] as f32 * alpha_f + background[0] as f32 * (1.0 - alpha_f)) as u8;
        let g = (foreground[1] as f32 * alpha_f + background[1] as f32 * (1.0 - alpha_f)) as u8;
        let b = (foreground[2] as f32 * alpha_f + background[2] as f32 * (1.0 - alpha_f)) as u8;

        Rgba([r, g, b, 255])
    }
}
