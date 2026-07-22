use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::mpsc,
};

use image::EncodableLayout;
use macroquad::{miniquad::window::set_window_size, prelude::*};
use notify::{Event, EventKind, RecursiveMode, Watcher, event::AccessKind};
use serde::{Deserialize, Serialize};

use crate::util::*;

mod util;

type Color16 = u16;
const HOR_LEN: usize = 40;
const CHUNKS_AMT: usize = 320 / HOR_LEN;

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
    fn get_size(&self) -> (u16, u16) {
        let t = format!("{self:?}");
        let t = t.split_once("_").unwrap().1.split_once("x").unwrap();
        (t.0.parse().unwrap(), t.1.parse().unwrap())
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

/// Checks if both bytes in the Color are the same
fn is_color_same_bytes(color: Color16) -> bool {
    let b = color.to_be_bytes();
    b[0] == b[1]
}

fn get_chunks_of_rect(y: u16, height: u16) -> Vec<usize> {
    let mut c = Vec::new();
    for chunk in 0..CHUNKS_AMT {
        if chunk * HOR_LEN >= y as usize && chunk * HOR_LEN <= (y + height) as usize {
            c.push(chunk);
        }
    }
    c
}

#[derive(Hash, PartialEq, Eq)]
struct Bounds {
    start: u16,
    end: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct CanvasData {
    items: Vec<UiItem>,
}
struct Chunk {
    base: String,
    per_pixel: String,
}
struct Canvas {
    data: CanvasData,
    camera: Camera2D,
    name: String,
    parent_path: PathBuf,
}
impl Canvas {
    #[allow(unused)]
    fn from_items(items: Vec<UiItem>, path: &PathBuf) -> Self {
        Self::new(CanvasData { items }, path)
    }
    fn new(data: CanvasData, path: &PathBuf) -> Self {
        let render_target = render_target(480, 320);
        render_target.texture.set_filter(FilterMode::Nearest);
        let camera = Camera2D {
            render_target: Some(render_target),
            zoom: Vec2::new(1.0 / 480.0 * 2.0, 1.0 / 320.0 * 2.0),
            target: Vec2::new(480.0 / 2.0, 320.0 / 2.0),
            ..Default::default()
        };
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let parent_path = path.parent().unwrap().to_path_buf();
        Self {
            data,
            camera,
            name,
            parent_path,
        }
    }
    fn generate_code(&self) -> String {
        let mut new = String::new();
        let mut setup = String::new();
        let mut imgs = 0;
        new += &format!("void {}DrawCallback(int i) {{\n", self.name);
        let mut multi_chunk_code: HashMap<Bounds, Chunk> = HashMap::new();

        fn insert_chunk_code(
            multi_chunk_code: &mut HashMap<Bounds, Chunk>,
            b: Bounds,
            value: &str,
            pixel: bool,
        ) {
            if multi_chunk_code.contains_key(&b) {
                if pixel {
                    multi_chunk_code.get_mut(&b).unwrap().per_pixel += &value;
                } else {
                    multi_chunk_code.get_mut(&b).unwrap().base += &value;
                }
            } else {
                let chunk = if pixel {
                    Chunk {
                        base: String::new(),
                        per_pixel: value.to_string(),
                    }
                } else {
                    Chunk {
                        base: value.to_string(),
                        per_pixel: String::new(),
                    }
                };
                multi_chunk_code.insert(b, chunk);
            }
        }

        for item in &self.data.items {
            match item {
                UiItem::Img(x, y, path) => {
                    let index = imgs;
                    imgs += 1;
                    setup += &format!(
                        "
FIL img{index};
FRESULT res = f_open(&img{index}, {path:?}, FA_READ);
if (res != FR_OK) {{
print(\"f_open failed with code: %d\r\n\", res);
return;
}}"
                    );
                    // get image referenced.
                    let img = imgtoibi::ibi_to_rgb(
                        &std::fs::read(PathBuf::from("filesystem").join(path)).unwrap(),
                    );
                    let height = img.height();
                    let width = img.width();

                    let c = get_chunks_of_rect(*y, height as _);
                    let bounds = Bounds {
                        start: c[0] as _,
                        end: c[c.len() - 1] as _,
                    };
                    let text = format!(
                        "int currentY = i*HOR_LEN+O;
                    if (currentY >= {y}) {{
                        res = f_lseek(&img{index}, (currentY-{y})*width*2);
                        if (res != FR_OK) {{
                            print(\"f_seek failed with code: %d\r\n\", res);
                        }}
                        res = f_read(&img{index},
                                    &disp_buf[(currentY-{y})*480*2+{x}*2],
                                    {width}*2, 0);
                        if (res != FR_OK) {{
                            print(\"f_read failed with code: %d\r\n\", res);
                        }}
                    }}"
                    );
                    insert_chunk_code(&mut multi_chunk_code, bounds, &text, true);
                }
                UiItem::Text(x, y, text, font_size, color) => {
                    let size = font_size.get_size();
                    let mut c = Vec::new();
                    for co in 0..CHUNKS_AMT {
                        let chunk_y = co * HOR_LEN;
                        if (chunk_y + 40 >= *y as usize
                            && chunk_y + 40 <= *y as usize + size.1 as usize)
                            || (chunk_y >= *y as usize && chunk_y <= *y as usize + size.1 as usize)
                        {
                            c.push(co);
                        }
                    }

                    insert_chunk_code(
                        &mut multi_chunk_code,
                        Bounds {
                            start: c[0] as _,
                            end: c[c.len() - 1] as _,
                        },
                        &format!(
                            "writeStringToBuffer({x}, {y}-o*HOR_LEN, {text:?}, {font_size:?}, 0x{color:04x}, disp_buf, 480, HOR_LEN);\n"
                        ),
                        false,
                    );
                }
                UiItem::Rect(x, y, w, h, color) => {
                    let c = get_chunks_of_rect(*y, *h);
                    let mut bounds = Bounds {
                        start: c[0] as _,
                        end: c[c.len() - 1] as _,
                    };
                    fn gen_code_for_chunk(
                        chunk_index: usize,
                        x: &u16,
                        y: &u16,
                        w: &u16,
                        h: &u16,
                        color: &Color16,
                    ) -> String {
                        let mut new = String::new();
                        let last = *y + *h - chunk_index as u16 * HOR_LEN as u16;
                        let is_last = last < 40;
                        let last = last.min(40);
                        if is_last {
                            new += &format!("if (o<{}) {{\n", last);
                        }
                        let memset = if is_color_same_bytes(*color) {
                            "memset"
                        } else {
                            "memset_u16"
                        };
                        new +=
                            &format!("{memset}(&disp_buf[o*480*2+{x}*2],0x{color:04x},{w}*2);\n");
                        if is_last {
                            new += &format!("for (int o=0; o<{};o++) {{\n", last);
                            new += "}\n";
                        }
                        new
                    }
                    let first_code = gen_code_for_chunk(bounds.start as _, x, y, w, h, color);
                    let last_code = gen_code_for_chunk(bounds.end as _, x, y, w, h, color);
                    if first_code != last_code {
                        let b = Bounds {
                            start: bounds.end,
                            end: bounds.end,
                        };
                        bounds.end -= 1;
                        insert_chunk_code(&mut multi_chunk_code, b, &last_code, true);
                    }
                    insert_chunk_code(&mut multi_chunk_code, bounds, &first_code, true);
                }
                UiItem::Base(color) => {
                    let memset = if is_color_same_bytes(*color) {
                        "memset"
                    } else {
                        "memset_u16"
                    };
                    new += "for (int o=0; o<40;o++) {\n";
                    new += &format!("{memset}(&disp_buf[o*480*2],0x{color:04x},480*2);\n");
                    new += "}\n";
                }
                _ => {}
            }
        }
        for (bounds, chunk) in multi_chunk_code.into_iter() {
            new += &format!("if (i >= {} && i <= {}) {{\n", bounds.start, bounds.end);
            new += &chunk.base;
            if !chunk.per_pixel.is_empty() {
                new += "for (int o = 0; o<40; o++) {\n";
                new += &chunk.per_pixel;
                new += "}\n";
            }
            new += &format!("}}\n");
        }
        new += "}\n";
        new += &format!(
            "
void {}Draw() {{
  {setup}
  for (int i = 0; i < 320/HOR_LEN; i++) {{
    {}DrawCallback(i);
  }}
}}\n",
            self.name, self.name
        );
        new
    }
    fn write_to_file(&self) {
        let mut new = String::new();
        new += "// autogenerated ui file by ittyOS-ui-designer. dont modify manually please <3\n\n";
        new += "/*\n";
        new += &serde_json::to_string_pretty(&self.data)
            .unwrap()
            .replace("  \"items\": [\n", "")
            .trim_start_matches("{")
            .replace("\n    ", "\n")
            .trim_end_matches("]\n}")
            .trim();
        new += "\n*/\n\n";
        new += &self.generate_code();

        std::fs::write(self.parent_path.join(self.name.to_string() + ".c"), new).unwrap();
        new = String::new();
        new += "// autogenerated ui file by ittyOS-ui-designer. dont modify manually please <3\n\n";
        new += &format!("void {}DrawCallback(int i);\n", self.name);
        new += &format!("void {}Draw();\n", self.name);
        std::fs::write(self.parent_path.join(self.name.to_string() + ".h"), new).unwrap();
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

                    // add alpha channel
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

fn parse_canvas_data(text: &str) -> Option<CanvasData> {
    let new = "{\"items\":[".to_string()
        + text
            .split_once("/*")
            .unwrap()
            .1
            .split_once("*/")
            .unwrap()
            .0
            .trim()
        + "]}";
    let res = serde_json::from_str(&new);
    res.ok()
}

fn window_conf() -> Conf {
    Conf {
        window_title: "ittyOS ui designer".to_string(),
        window_width: 480,
        window_height: 320,
        ..Default::default()
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    set_window_size(
        (480.0 * screen_dpi_scale()) as _,
        (320.0 * screen_dpi_scale()) as _,
    );
    println!("ittyOS ui designer v{}", env!("CARGO_PKG_VERSION"));

    let Some(path) = std::env::args().nth(1) else {
        println!("no path given. usage: `ittyOS-ui-designer <file>`");
        return;
    };
    let path = PathBuf::from(&path);
    let mut canvas = if !path.exists() {
        Canvas::from_items(vec![], &path)
    } else {
        Canvas::new(
            parse_canvas_data(&std::fs::read_to_string(&path).unwrap()).unwrap(),
            &path,
        )
    };

    canvas.write_to_file();

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx).unwrap();
    watcher
        .watch(Path::new(&path), RecursiveMode::Recursive)
        .unwrap();

    loop {
        if let Ok(Ok(r)) = rx.try_recv() {
            while let Ok(_) = rx.try_recv() {}
            if let EventKind::Access(AccessKind::Close(_)) = r.kind {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Some(data) = parse_canvas_data(&text) {
                        canvas.data = data;
                    }
                }
            }
        }
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
