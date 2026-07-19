use macroquad::prelude::*;

type Color = u16;

enum UiItem {
    /// color
    Base(Color),
    /// x, y, width, height, color
    Rect(u16, u16, u16, u16, Color),
    /// x, y, text, color
    Text(u16, u16, String, Color),
    /// x, y, path
    Img(u16, u16, String),
    /// x, y, path, cropx, cropy, crop width, crop height
    ImgEx(u16, u16, String, u16, u16, u16, u16),
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
        for item in ui.iter() {
            match item {
                _ => todo!(),
            }
        }
        next_frame().await;
    }
}
