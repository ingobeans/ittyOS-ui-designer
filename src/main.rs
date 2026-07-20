use image::EncodableLayout;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::util::*;

mod util;

type Color16 = u16;

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug)]
enum FontSize {
    Font_7x10,
    Font_11x18,
    Font_16x26,
}
impl FontSize {
    fn get_preview_size(&self) -> f32 {
        match self {
            FontSize::Font_7x10 => 7.0,
            FontSize::Font_11x18 => 18.0,
            FontSize::Font_16x26 => 26.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum UiItem {
    /// color
    Base(Color16),
    /// x, y, width, height, color
    Rect(u16, u16, u16, u16, Color16),
    /// x, y, text, size, font, color
    Text(u16, u16, String, FontSize, Color16),
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

#[derive(Serialize, Deserialize, Debug)]
struct CanvasData {
    items: Vec<UiItem>,
}
struct Canvas {
    data: CanvasData,
    camera: Camera2D,
}
impl Canvas {
    fn new(items: Vec<UiItem>) -> Self {
        let render_target = render_target(480, 320);
        render_target.texture.set_filter(FilterMode::Nearest);
        let camera = Camera2D {
            render_target: Some(render_target),
            zoom: Vec2::new(1.0 / 480.0 * 2.0, 1.0 / 320.0 * 2.0),
            target: Vec2::new(480.0 / 2.0, 320.0 / 2.0),
            ..Default::default()
        };
        Self {
            data: CanvasData { items },
            camera,
        }
    }
    fn render(&self) {
        set_camera(&self.camera);
        gl_use_material(&GRID_MATERIAL);
        draw_rectangle(0.0, 0.0, 480.0, 320.0, WHITE);
        gl_use_default_material();

        for item in self.data.items.iter() {
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
                #[allow(unused)]
                UiItem::Img(x, y, path) | UiItem::ImgEx(x, y, path, ..) => {
                    let mut crop_x = None;
                    let mut crop_y = None;
                    let mut crop_width = None;
                    let mut crop_height = None;

                    if let UiItem::ImgEx(x, y, path, _crop_x, _crop_y, _crop_width, _crop_height) =
                        item
                    {
                        crop_x = Some(*_crop_x);
                        crop_y = Some(*_crop_x);
                        crop_width = Some(*_crop_x);
                        crop_height = Some(*_crop_x);
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
                UiItem::Text(x, y, text, font, color) => {
                    draw_text(
                        text,
                        *x as f32,
                        *y as f32 + font.get_preview_size() / 2.0,
                        font.get_preview_size(),
                        u16_to_color(*color),
                    );
                }
            }
        }
        set_default_camera();
    }
}

#[macroquad::main("ittyOS ui designer")]
async fn main() {
    println!("ittyOS ui designer v{}", env!("CARGO_PKG_VERSION"));
    let canvas = Canvas::new(vec![
        //UiItem::Base(0),
        UiItem::Img(0, 0, "images/cat.ibi".to_string()),
        UiItem::Rect(0, 0, 52, 320, 0x6529),
        UiItem::Rect(480 - 52, 0, 52, 320, 0x6529),
        UiItem::Text(0, 0, "wahoo".to_string(), FontSize::Font_16x26, 0xffff),
    ]);
    let string = serde_json::to_string(&canvas.data).unwrap();
    println!("{}", string);

    loop {
        canvas.render();
        draw_texture(
            &canvas.camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
        );
        next_frame().await;
    }
}
