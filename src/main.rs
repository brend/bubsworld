use macroquad::{
    color::*,
    shapes::{draw_circle, draw_rectangle},
    text::draw_text,
    window::{clear_background, next_frame, screen_height, screen_width},
};
use neural_network_study::NeuralNetwork;
use rand::prelude::*;

const R: f32 = 3.0;

#[macroquad::main("Game and Watch")]
async fn main() {
    let mut rng = rand::rngs::StdRng::from_os_rng();
    let mut population = Population::new(100, &mut rng);
    let mut danger_zones = vec![];
    let max_time = 1100;
    let mut time = 0;
    let mut last_generation_survival_rate = 0;
    loop {
        clear_background(DARKGRAY);

        if time >= max_time {
            let survivor_count = population.bubs.iter().filter(|b| b.is_alive).count();
            last_generation_survival_rate =
                (100.0 * survivor_count as f32 / population.bubs.len() as f32) as usize;
            population = population.spawn_next_generation(&mut rng);
            danger_zones.clear();
            time = 0;
        }

        if time == 1000 {
            danger_zones.push(DangerZone::new(
                0.0,
                0.0,
                screen_width() * 0.5,
                screen_height(),
            ));
        }

        population.update(&danger_zones);
        population.draw();

        for dz in &danger_zones {
            dz.draw();
        }

        draw_text(&format!("time: {}", time), 20.0, 20.0, 20.0, WHITE);
        draw_text(
            &format!("survival rate: {}", last_generation_survival_rate),
            20.0,
            46.0,
            20.0,
            WHITE,
        );
        time += 1;
        next_frame().await;
    }
}

struct Bub {
    x: f32,
    y: f32,
    brain: NeuralNetwork,
    is_alive: bool,
}

impl Bub {
    fn new(x: f32, y: f32, rng: &mut StdRng) -> Self {
        let nn = NeuralNetwork::new(3, 6, 4, Some(rng));
        Bub {
            x,
            y,
            brain: nn,
            is_alive: true,
        }
    }

    fn new_with_brain(x: f32, y: f32, brain: NeuralNetwork) -> Self {
        Bub {
            x,
            y,
            brain,
            is_alive: true,
        }
    }

    fn update(&mut self, input: &Input) {
        let output = Action::new(self.brain.predict(input.to_vec()));

        match output {
            Action::Left => self.x -= 1.0,
            Action::Right => self.x += 1.0,
            Action::Up => self.y -= 1.0,
            Action::Down => self.y += 1.0,
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
    Left,
    Right,
    Up,
    Down,
}

impl Action {
    fn new(values: Vec<f64>) -> Self {
        let action_index = values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        match action_index {
            0 => Action::Left,
            1 => Action::Right,
            2 => Action::Up,
            3 => Action::Down,
            _ => panic!("invalid output"),
        }
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
            let i = Input {
                x: b.x / screen_width(),
                y: b.y / screen_height(),
                age: 0.0,
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
        // Eliminate the expired bubs
        self.bubs.retain(|b| b.is_alive);
        // Shuffle the remaining ones
        self.bubs.shuffle(rng);
        // Create the new population from randomly mutated survivors
        let mut next_generation = vec![];
        while next_generation.len() < size {
            let mut child_brain = self.bubs[rng.random_range(0..self.bubs.len())]
                .brain
                .clone();
            child_brain.mutate(rng, 0.1);
            let child = Bub::new_with_brain(
                rng.random_range(0.0..screen_width()),
                rng.random_range(0.0..screen_height()),
                child_brain,
            );
            next_generation.push(child);
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
