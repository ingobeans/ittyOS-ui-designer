use macroquad::prelude::*;

#[macroquad::main("ittyOS ui designer")]
async fn main() {
    println!("ittyOS ui designer v{}", env!("CARGO_PKG_VERSION"));
    loop {
        next_frame().await;
    }
}
