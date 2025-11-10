use image::{Rgba, RgbaImage, ImageEncoder, ExtendedColorType};
use rusttype::{Font, Scale, point};
use rand::Rng;
use std::sync::Arc;
use tracing::info;

use crate::models::generate::{GenerateRequest, GenerateError};

const TEMPLATE_PATH: &str = "assets/template.png";
const FONT_PATH: &str = "assets/font.ttf";

#[derive(Debug, Clone)]
struct FieldPosition {
    x: f32,
    y: f32
}

#[derive(Debug)]
pub struct ImageGenerator {
    template: Arc<RgbaImage>,
    font: Arc<Font<'static>>,
    fields: std::collections::HashMap<&'static str, Vec<FieldPosition>>,
}

impl ImageGenerator {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let template_image = image::open(TEMPLATE_PATH)
            .map_err(|e| format!("Failed to open template image: {}", e))?;
        let template = template_image.to_rgba8();

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
                FieldPosition { x: 145.0, y: 730.0 },
                FieldPosition { x: 265.0, y: 1020.0 },
            ]),
            ("year", vec![
                FieldPosition { x: 367.0, y: 352.0 },
                FieldPosition { x: 676.0, y: 453.0 },
                FieldPosition { x: 392.0, y: 843.0 },
                FieldPosition { x: 483.0, y: 1096.0 },
                FieldPosition { x: 836.0, y: 1096.0 },
            ]),
            ("time", vec![
                FieldPosition { x: 753.0, y: 457.0 },
            ]),
        ]);

        Ok(Self {
            template: Arc::new(template),
            font: Arc::new(font),
            fields,
        })
    }

    pub async fn generate_image(
        &self,
        request: &GenerateRequest,
    ) -> Result<Vec<u8>, GenerateError> {
        let mut rng = rand::rng();
        let number = rng.random_range(64*64..512*512);

        // Create a copy of the template to work with
        let mut image = self.template.as_ref().clone();

        // Draw text fields - batch operations to reduce image cloning
        self.draw_all_text(&mut image, request, number)
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

    fn draw_all_text(
        &self,
        image: &mut RgbaImage,
        request: &GenerateRequest,
        number: u32,
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
                let x = position.x + rng.random_range(-2.0..4.0);
                let y = position.y + rng.random_range(-2.0..2.0);
                self.draw_text_at_position(image, &request.address, x, y, Scale::uniform(rng.random_range(26.0..34.0)), color)?;
            }
        }

        // Draw issuer
        if let Some(positions) = self.fields.get("issuer") {
            for position in positions {
                let x = position.x + rng.random_range(-2.0..3.0);
                let y = position.y + rng.random_range(-2.0..2.0);
                self.draw_text_at_position(image, &request.issuer, x, y, Scale::uniform(rng.random_range(26.0..38.0)), color)?;
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

        // Draw year
        if let Some(positions) = self.fields.get("year") {
            for position in positions {
                let x = position.x + rng.random_range(-1.7..1.7);
                let y = position.y + rng.random_range(-1.2..1.2);
                self.draw_text_at_position(image, "25", x, y, Scale::uniform(rng.random_range(32.0..37.0)), color)?;
            }
        }

        // Draw time
        if let Some(positions) = self.fields.get("time") {
            for position in positions {
                let x = position.x + rng.random_range(-2.0..8.0);
                let y = position.y + rng.random_range(-2.2..2.2);
                self.draw_text_at_position(image, "12:34", x, y, Scale::uniform(rng.random_range(27.0..34.0)), color)?;
            }
        }


        Ok(())
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
