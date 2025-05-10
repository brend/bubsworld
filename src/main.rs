use macroquad::{
    color::*,
    shapes::{draw_circle, draw_rectangle},
    text::draw_text,
    window::{clear_background, next_frame, screen_height, screen_width},
};
use rand::prelude::*;

const R: f32 = 3.0;

#[macroquad::main("Game and Watch")]
async fn main() {
    let mut rng = rand::rng();
    let mut p = Population::new(100, &mut rng);
    let mut danger_zones = vec![];
    let mut time = 0;
    loop {
        clear_background(DARKGRAY);

        if time == 1000 {
            danger_zones.push(DangerZone::new(
                0.0,
                0.0,
                screen_width() * 0.5,
                screen_height(),
            ));
        }

        p.update(&danger_zones);
        p.draw();

        for dz in &danger_zones {
            dz.draw();
        }

        draw_text(&format!("time: {}", time), 20.0, 20.0, 20.0, WHITE);
        time += 1;
        next_frame().await;
    }
}

struct Bub {
    x: f32,
    y: f32,
}

impl Bub {
    fn new(x: f32, y: f32) -> Self {
        Bub { x, y }
    }

    fn update(&mut self, _input: &Input) {
        self.x += 1.0;
    }

    fn draw(&self) {
        draw_circle(self.x, self.y, R, BLUE);
    }
}

struct Input {
    x: f32,
    y: f32,
    age: f32,
}

struct Output {
    move_left: f32,
    move_right: f32,
    move_up: f32,
    move_down: f32,
}

struct Population {
    bubs: Vec<Bub>,
}

impl Population {
    fn new(size: usize, rng: &mut ThreadRng) -> Self {
        let mut bubs = vec![];
        for _ in 0..size {
            bubs.push(Bub::new(
                rng.random_range(0.0..screen_width()),
                rng.random_range(0.0..screen_height()),
            ));
        }
        Population { bubs }
    }

    fn update(&mut self, danger_zones: &[DangerZone]) {
        for b in self.bubs.iter_mut() {
            let i = Input {
                x: b.x / screen_width(),
                y: b.y / screen_height(),
                age: 0.0,
            };
            b.update(&i);
        }

        self.bubs
            .retain(|b| !danger_zones.iter().any(|dz| dz.contains(b.x, b.y)));
    }

    fn draw(&self) {
        for b in &self.bubs {
            b.draw();
        }
    }
}

struct DangerZone {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl DangerZone {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        DangerZone { x, y, w, h }
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }

    fn draw(&self) {
        draw_rectangle(
            self.x,
            self.y,
            self.w,
            self.h,
            Color::from_rgba(255, 0, 0, 80),
        );
    }
}
