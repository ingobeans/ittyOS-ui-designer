use image::{DynamicImage, EncodableLayout, ImageBuffer};
use macroquad::prelude::*;

type Color16 = u16;

enum UiItem {
    /// color
    Base(Color16),
    /// x, y, width, height, color
    Rect(u16, u16, u16, u16, Color16),
    /// x, y, text, color
    Text(u16, u16, String, Color16),
    /// x, y, path
    Img(u16, u16, String),
    /// x, y, path, cropx, cropy, crop width, crop height
    ImgEx(u16, u16, String, u16, u16, u16, u16),
}

fn u16_to_color(pixel: Color16) -> Color {
    let bytes = pixel.to_le_bytes();
    let pixel: u16 = (bytes[0] as u16) << 8 | bytes[1] as u16;
    let r = ((pixel & 0b1111100000000000) >> 11) as u8 * 8;
    let g = ((pixel & 0b0000011111100000) >> 5) as u8 * 4;
    let b = (pixel & 0b0000000000011111) as u8 * 8;
    Color::new(r as f32 / 256.0, g as f32 / 256.0, b as f32 / 256.0, 1.0)
}

#[macroquad::main("ittyOS ui designer")]
async fn main() {
    println!("ittyOS ui designer v{}", env!("CARGO_PKG_VERSION"));
    let mut ui = vec![
        UiItem::Base(0),
        UiItem::Img(0, 0, "images/cat.ibi".to_string()),
        UiItem::Rect(0, 0, 52, 320, 0x6529),
        UiItem::Rect(480 - 52, 0, 52, 320, 0x6529),
        UiItem::Text(0, 0, "wahoo".to_string(), 1),
    ];
    loop {
        draw_rectangle(0.0, 0.0, 480.0, 320.0, WHITE);

        for item in ui.iter() {
            match item {
                UiItem::Base(color) => {
                    draw_rectangle(0.0, 0.0, 480.0, 320.0, u16_to_color(*color));
                }
                UiItem::Rect(x, y, width, height, color) => {
                    draw_rectangle(
                        *x as f32,
                        *y as f32,
                        *width as f32,
                        *height as f32,
                        u16_to_color(*color),
                    );
                }
                UiItem::Img(x, y, path) | UiItem::ImgEx(x, y, path, ..) => {
                    let mut crop_x = None;
                    let mut crop_y = None;
                    let mut crop_width = None;
                    let mut crop_height = None;

                    match item {
                        UiItem::ImgEx(x, y, path, _crop_x, _crop_y, _crop_width, _crop_height) => {
                            crop_x = Some(*_crop_x);
                            crop_y = Some(*_crop_x);
                            crop_width = Some(*_crop_x);
                            crop_height = Some(*_crop_x);
                        }
                        _ => {}
                    }
                    let buffer = imgtoibi::ibi_to_rgb(
                        &std::fs::read("filesystem/".to_string() + path).unwrap(),
                    );
                    let w = buffer.width();
                    let h = buffer.height();
                    let bytes = buffer.as_bytes().to_vec();
                    let mut new = Vec::new();
                    for b in bytes.chunks(3) {
                        new.extend_from_slice(b);
                        new.push(255);
                    }
                    let image = Image {
                        width: w as u16,
                        height: h as u16,
                        bytes: new,
                    };
                    let texture = Texture2D::from_image(&image);
                    draw_texture(&texture, *x as f32, *y as f32, WHITE);
                }
                _ => {}
            }
        }
        next_frame().await;
    }
}
