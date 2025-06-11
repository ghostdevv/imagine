use gif::{Decoder, Encoder, Frame, Repeat};
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::filter::gaussian_blur_f32;
use rusttype::{Font, Scale};
use std::io::Cursor;
use text_on_image::{FontBundle, TextJustify, VerticalAnchor, WrapBehavior};
use worker::{Error, Result};

fn replace_emojis_with_text(text: &str) -> String {
    text.chars()
        .map(|c| {
            let replacement = match c {
                '😀' | '😃' | '😄' => ":)",
                '😢' | '😭' => ":(",
                '😎' => "B)",
                '❤' | '💕' | '💖' => "<3",
                '👍' => "+1",
                '👎' => "-1",
                '🎉' | '🎊' => "*",
                '✨' => "*",
                '🔥' => "fire",
                '💯' => "100",
                '👋' => "wave",
                c if c as u32 >= 0x1F300 && c as u32 <= 0x1F9FF => "?",
                _ => return c.to_string(),
            };
            replacement.to_string()
        })
        .collect()
}

pub fn add_text_to_gif(gif_data: &[u8], text: &str) -> Result<Vec<u8>> {
    // Replace emojis with text representations
    let display_text = replace_emojis_with_text(text);
    
    // Decode the input GIF
    let mut decoder =
        Decoder::new(Cursor::new(gif_data)).map_err(|e| Error::from(e.to_string()))?;

    // Get the global color table if it exists
    let global_palette = decoder.global_palette().map(|p| p.to_vec());

    // Prepare output buffer
    let mut output = Vec::new();

    // Create encoder with the same dimensions
    let width = decoder.width();
    let height = decoder.height();
    let mut encoder =
        Encoder::new(&mut output, width, height, &[]).map_err(|e| Error::from(e.to_string()))?;
    encoder
        .set_repeat(Repeat::Infinite)
        .map_err(|e| Error::from(e.to_string()))?;

    // Load font
    let font_data = Vec::from(include_bytes!("./roboto.ttf"));
    let font = Font::try_from_vec(font_data).ok_or_else(|| Error::from("Failed to load font"))?;

    // Calculate text position (centered horizontally, near top)
    let text_x = width as i32 / 2;
    let text_y = 22;

    // Process each frame
    let mut frame_number = 0;
    while let Some(frame) = decoder
        .read_next_frame()
        .map_err(|e| Error::from(e.to_string()))?
    {
        // Get the palette for this frame
        let palette = frame
            .palette
            .as_ref()
            .or(global_palette.as_ref())
            .ok_or_else(|| Error::from("No palette found"))?;

        // Convert frame to RGBA image
        let rgba_image = frame_to_rgba(&frame, palette, width, height);

        // Convert to DynamicImage for text_on_image
        let mut dynamic_image = DynamicImage::ImageRgba8(rgba_image);

        // Calculate alpha to fade in starting after frame 8, over 10 frames
        let alpha = if frame_number <= 8 {
            0
        } else if frame_number < 29 {
            // Use gentler ease-in curve for earlier visibility
            let progress = (frame_number - 8) as f32 / 20.0;
            let eased_progress = progress.powf(0.7); // Gentler curve for earlier visibility
            (eased_progress * 255.0) as u8
        } else {
            255
        };

        // Always try to draw text (even if very faint)
        let final_image = if alpha == 0 {
            dynamic_image.to_rgba8()
        } else {
            // Create font bundles with normal colors
            let outline_font_bundle = FontBundle::new(
                &font,
                Scale { x: 30.0, y: 30.0 },
                Rgba([0, 0, 0, 255]), // Black outline
            );

            let text_font_bundle = FontBundle::new(
                &font,
                Scale { x: 30.0, y: 30.0 },
                Rgba([255, 255, 255, 255]), // White text
            );
            
            // Save the original image before drawing text
            let original_image = dynamic_image.clone();

            // Draw outline by applying black text with offsets
            let outline_offsets = [
            (-2, -2),
            (-1, -2),
            (0, -2),
            (1, -2),
            (2, -2),
            (-2, -1),
            (-1, -1),
            (0, -1),
            (1, -1),
            (2, -1),
            (-2, 0),
            (-1, 0),
            (1, 0),
            (2, 0),
            (-2, 1),
            (-1, 1),
            (0, 1),
            (1, 1),
            (2, 1),
            (-2, 2),
            (-1, 2),
            (0, 2),
            (1, 2),
            (2, 2),
        ];

        for (dx, dy) in outline_offsets.iter() {
            text_on_image::text_on_image(
                &mut dynamic_image,
                &display_text,
                &outline_font_bundle,
                text_x + dx,
                text_y + dy,
                TextJustify::Center,
                VerticalAnchor::Top,
                WrapBehavior::NoWrap,
            );
        }

        // Draw main text on top
        text_on_image::text_on_image(
            &mut dynamic_image,
            &display_text,
            &text_font_bundle,
            text_x,
            text_y,
            TextJustify::Center,
            VerticalAnchor::Top,
            WrapBehavior::NoWrap,
        );

            // Convert images to RGBA
            let text_image = dynamic_image.to_rgba8();
            let original_rgba = original_image.to_rgba8();
            
            // For frames during fade-in, we'll apply blur
            if alpha < 255 {
            // Create a mask for the text
            let mut text_mask = RgbaImage::new(width as u32, height as u32);
            for y in 0..height as u32 {
                for x in 0..width as u32 {
                    let text_pixel = text_image.get_pixel(x, y);
                    let original_pixel = original_rgba.get_pixel(x, y);
                    
                    // If pixels differ, it's text - make it white in the mask
                    if text_pixel != original_pixel {
                        text_mask.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                    } else {
                        text_mask.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                    }
                }
            }
            
            // Apply blur to the mask based on alpha (more blur = less visible)
            // Use exponential curve for smoother fade-in at the start
            let alpha_normalized = alpha as f32 / 255.0;
            let blur_amount = (1.0 - alpha_normalized).powf(1.2) * 25.0;
            let blurred_mask = if blur_amount > 0.1 {
                gaussian_blur_f32(&text_mask, blur_amount)
            } else {
                text_mask
            };
            
            // Blend images based on blurred mask
            let mut blended_image = RgbaImage::new(width as u32, height as u32);
            for y in 0..height as u32 {
                for x in 0..width as u32 {
                    let mask_value = blurred_mask.get_pixel(x, y)[0] as f32 / 255.0;
                    let text_pixel = text_image.get_pixel(x, y);
                    let original_pixel = original_rgba.get_pixel(x, y);
                    
                    // Boost visibility more aggressively in early frames
                    let boosted_alpha = (alpha_normalized * 3.0).min(1.0);
                    let visibility = mask_value * boosted_alpha;
                    
                    // Interpolate between original and text based on mask and visibility
                    let r = (original_pixel[0] as f32 * (1.0 - visibility) + text_pixel[0] as f32 * visibility) as u8;
                    let g = (original_pixel[1] as f32 * (1.0 - visibility) + text_pixel[1] as f32 * visibility) as u8;
                    let b = (original_pixel[2] as f32 * (1.0 - visibility) + text_pixel[2] as f32 * visibility) as u8;
                    
                    blended_image.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
                blended_image
            } else {
                // Full alpha - just use the text image as-is
                text_image
            }
        };

        // Convert RGBA image back to indexed color for GIF, preserving original palette
        let (indexed_buffer, palette) = rgba_to_indexed(&final_image, width, height, palette)?;

        // Create new frame
        let mut new_frame = Frame::default();
        new_frame.delay = frame.delay;
        new_frame.dispose = frame.dispose;
        new_frame.transparent = frame.transparent;
        new_frame.needs_user_input = frame.needs_user_input;
        new_frame.top = 0;
        new_frame.left = 0;
        new_frame.width = width;
        new_frame.height = height;
        new_frame.interlaced = false;
        new_frame.palette = Some(palette);
        new_frame.buffer = indexed_buffer.into();

        // Write frame to encoder
        encoder
            .write_frame(&new_frame)
            .map_err(|e| Error::from(e.to_string()))?;

        frame_number += 1;
    }

    drop(encoder);
    Ok(output)
}

fn frame_to_rgba(frame: &Frame, palette: &[u8], width: u16, height: u16) -> RgbaImage {
    let mut image = RgbaImage::new(width as u32, height as u32);

    for y in 0..frame.height {
        for x in 0..frame.width {
            let pixel_index = y as usize * frame.width as usize + x as usize;
            if pixel_index < frame.buffer.len() {
                let color_index = frame.buffer[pixel_index] as usize;
                let palette_offset = color_index * 3;

                if palette_offset + 2 < palette.len() {
                    let r = palette[palette_offset];
                    let g = palette[palette_offset + 1];
                    let b = palette[palette_offset + 2];

                    // Handle transparency
                    let alpha = if Some(color_index as u8) == frame.transparent {
                        0
                    } else {
                        255
                    };

                    let img_x = frame.left + x;
                    let img_y = frame.top + y;

                    if img_x < width && img_y < height {
                        image.put_pixel(img_x as u32, img_y as u32, Rgba([r, g, b, alpha]));
                    }
                }
            }
        }
    }

    image
}

fn rgba_to_indexed(
    image: &RgbaImage,
    width: u16,
    height: u16,
    original_palette: &[u8],
) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut colors: Vec<[u8; 3]> = Vec::new();
    let mut color_map = std::collections::HashMap::new();

    // Start with the original palette colors
    let palette_colors = original_palette.len() / 3;
    for i in 0..palette_colors.min(256) {
        let offset = i * 3;
        if offset + 2 < original_palette.len() {
            let color = [
                original_palette[offset],
                original_palette[offset + 1],
                original_palette[offset + 2],
            ];
            colors.push(color);
            color_map.insert(color, i);
        }
    }

    let mut indexed = Vec::new();

    // Map each pixel to the closest color in the palette
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x as u32, y as u32);

            // Skip transparent pixels
            if pixel[3] < 128 {
                indexed.push(0); // Use first color for transparent pixels
                continue;
            }

            let color = [pixel[0], pixel[1], pixel[2]];

            // Check if exact color exists
            let index = if let Some(&idx) = color_map.get(&color) {
                idx
            } else {
                // Find nearest color in existing palette
                find_nearest_color(&colors, &color)
            };

            indexed.push(index as u8);
        }
    }

    // Convert to GIF palette format (reuse original palette)
    let palette = original_palette.to_vec();

    Ok((indexed, palette))
}

fn find_nearest_color(colors: &[[u8; 3]], target: &[u8; 3]) -> usize {
    let mut min_distance = u32::MAX;
    let mut best_index = 0;

    for (index, color) in colors.iter().enumerate() {
        // Use weighted color distance for better perceptual matching
        // Human eyes are more sensitive to green, then red, then blue
        let dr = (color[0] as i32 - target[0] as i32).abs() as u32;
        let dg = (color[1] as i32 - target[1] as i32).abs() as u32;
        let db = (color[2] as i32 - target[2] as i32).abs() as u32;

        // Weighted distance: R*30%, G*59%, B*11%
        let distance = (dr * dr * 3 + dg * dg * 6 + db * db) / 10;

        if distance < min_distance {
            min_distance = distance;
            best_index = index;
        }
    }

    best_index
}
