use macroquad::prelude::*;

#[macroquad::main("Game and Watch")]
async fn main() {
    loop {
        clear_background(DARKGRAY);
        next_frame().await;
    }
}
