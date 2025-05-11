use macroquad::{
    color::*,
    input::{KeyCode, is_key_pressed},
    shapes::{draw_circle, draw_rectangle},
    text::draw_text,
    window::{clear_background, next_frame, screen_height, screen_width},
};
use neural_network_study::NeuralNetwork;
use rand::prelude::*;

const R: f32 = 5.0;
const MAX_TIME: i32 = 2600;
const ELITE: f32 = 0.05;
const MUTATION_RATE: f64 = 0.03;

#[macroquad::main("Bubs World")]
async fn main() {
    let mut rng = rand::rngs::StdRng::from_os_rng();
    let mut population = Population::new(100, &mut rng);
    let mut danger_zones = vec![];
    let mut time = 0;
    let mut last_generation_survival_rate = 0;
    let mut iterations_per_frame = 1;
    let mut generation = 1;
    loop {
        clear_background(BLACK);

        if is_key_pressed(KeyCode::S) {
            iterations_per_frame += 1;
        } else if is_key_pressed(KeyCode::A) {
            iterations_per_frame -= 1;
        } else if is_key_pressed(KeyCode::Key0) {
            iterations_per_frame = 1;
        } else if is_key_pressed(KeyCode::Key9) {
            iterations_per_frame = 100;
        }

        iterations_per_frame = iterations_per_frame.clamp(1, 100);

        for _ in 0..iterations_per_frame {
            if time >= MAX_TIME {
                let survivor_count = population.bubs.iter().filter(|b| b.is_alive).count();
                last_generation_survival_rate =
                    (100.0 * survivor_count as f32 / population.bubs.len() as f32) as usize;
                population = population.spawn_next_generation(&mut rng);
                danger_zones.clear();
                time = 0;
                generation += 1;
            }

            if time == 500 {
                danger_zones.push(DangerZone::new(
                    0.0,
                    0.0,
                    screen_width() * 0.3,
                    screen_height() * 0.3,
                ));
            }

            match time {
                t if t > 2000 => {
                    danger_zones[0].y -= (screen_height() - danger_zones[0].h) / 500.0 + 0.01
                }
                t if t > 1500 => {
                    danger_zones[0].x -= (screen_width() - danger_zones[0].w) / 500.0 + 0.01
                }
                t if t > 1000 => {
                    danger_zones[0].y += (screen_height() - danger_zones[0].h) / 500.0 + 0.01
                }
                t if t > 500 => {
                    danger_zones[0].x += ((screen_width() - danger_zones[0].w) / 500.0) + 0.01;
                }
                _ => (),
            }

            population.update(&danger_zones);

            time += 1;
        }

        population.draw();

        for dz in &danger_zones {
            dz.draw();
        }

        draw_text(&format!("time: {}", time), 20.0, 20.0, 20.0, WHITE);
        draw_text(
            &format!("generation: {}", generation),
            20.0,
            46.0,
            20.0,
            WHITE,
        );
        if generation > 1 {
            draw_text(
                &format!(
                    "last generation's survival rate: {}",
                    last_generation_survival_rate
                ),
                20.0,
                72.0,
                20.0,
                WHITE,
            );
        }
        next_frame().await;
    }
}

struct Bub {
    x: f32,
    y: f32,
    brain: NeuralNetwork,
    is_alive: bool,
    age: usize,
}

impl Bub {
    fn new(x: f32, y: f32, rng: &mut StdRng) -> Self {
        let nn = NeuralNetwork::new(3, 6, 4, Some(rng));
        Bub {
            x,
            y,
            brain: nn,
            is_alive: true,
            age: 0,
        }
    }

    fn new_with_brain(x: f32, y: f32, brain: NeuralNetwork) -> Self {
        Bub {
            x,
            y,
            brain,
            is_alive: true,
            age: 0,
        }
    }

    fn update(&mut self, input: &Input) {
        self.age += 1;
        let output = Action::new(self.brain.predict(input.to_vec()));

        match output {
            Action::Move(dx, dy) => {
                self.x += dx as f32;
                self.y += dy as f32;
            }
        }

        self.x = self.x.clamp(0.0, screen_width());
        self.y = self.y.clamp(0.0, screen_height());
    }

    fn draw(&self) {
        if self.is_alive {
            draw_circle(self.x, self.y, R, BLUE);
        }
    }
}

struct Input {
    x: f32,
    y: f32,
    age: f32,
}

impl Input {
    fn to_vec(&self) -> Vec<f64> {
        vec![self.x as f64, self.y as f64, self.age as f64]
    }
}

enum Action {
    Move(i32, i32),
}

impl Action {
    fn new(values: Vec<f64>) -> Self {
        let dx = if values[0] > values[1] { -1 } else { 1 };
        let dy = if values[2] > values[3] { -1 } else { 1 };
        Action::Move(dx, dy)
    }
}

struct Population {
    bubs: Vec<Bub>,
}

impl Population {
    fn new(size: usize, rng: &mut StdRng) -> Self {
        let mut bubs = vec![];
        for _ in 0..size {
            bubs.push(Bub::new(
                rng.random_range(0.0..screen_width()),
                rng.random_range(0.0..screen_height()),
                rng,
            ));
        }
        Population { bubs }
    }

    fn update(&mut self, danger_zones: &[DangerZone]) {
        for b in self.bubs.iter_mut() {
            if !b.is_alive {
                continue;
            }

            let i = Input {
                x: b.x / screen_width(),
                y: b.y / screen_height(),
                age: b.age as f32 / MAX_TIME as f32,
            };
            b.update(&i);

            if danger_zones.iter().any(|dz| dz.contains(b.x, b.y)) {
                b.is_alive = false;
            }
        }
    }

    fn draw(&self) {
        for b in &self.bubs {
            b.draw();
        }
    }

    fn spawn_next_generation(&mut self, rng: &mut StdRng) -> Population {
        let size = self.bubs.len();
        // Keep only the longest surviving bubs
        self.bubs.sort_by(|b1, b2| b2.age.cmp(&b1.age));
        let retain_count = (ELITE * self.bubs.len() as f32).ceil() as usize;
        self.bubs.truncate(retain_count);
        // Create the new population from randomly mutated survivors
        let mut next_generation = vec![];
        let score_sum: f32 = self.bubs.iter().map(|b| b.age as f32).sum();
        while next_generation.len() < size {
            let r = rng.random_range(0.0..score_sum);
            let mut sum = 0.0;
            for b in &self.bubs {
                sum += b.age as f32;
                if sum >= r {
                    let mut child_brain = self.bubs[rng.random_range(0..self.bubs.len())]
                        .brain
                        .clone();
                    child_brain.mutate(rng, MUTATION_RATE);
                    let child = Bub::new_with_brain(
                        rng.random_range(0.0..screen_width()),
                        rng.random_range(0.0..screen_height()),
                        child_brain,
                    );
                    next_generation.push(child);
                    break;
                }
            }
        }
        Population {
            bubs: next_generation,
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
