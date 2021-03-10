use js_sys::Math::random as js_math_random;
use std::f64;
use std::fmt::{self, Display};

const MIN_POS: f64 = 0.0;

#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

// not sure what I'm going to do with this but currently leaving in option to use either
fn random() -> f64 {
    if false {
        js_math_random()
    } else {
        rand::random()
    }
}

fn random_in_range(min: f64, max: f64) -> f64 {
    (random() * (max - min)) + min
}

fn saturating_random_in_range(current: f64, delta: f64, min: f64, max: f64) -> f64 {
    let min = f64::max(min, current - delta);
    let max = f64::min(max, current + delta);
    random_in_range(min, max)
}

impl Position {
    fn new(config: &CircleConfig) -> Self {
        Position {
            x: random_in_range(MIN_POS, config.width),
            y: random_in_range(MIN_POS, config.height),
        }
    }

    fn update(&mut self, config: &CircleConfig, _radius: f64) {
        // let max_position_delta = (100.0 - radius) * config.max_position_delta.powi(2)
        //     + (2. * radius - 100.0) * config.max_position_delta;
        // let max_position_delta = (2. * radius).powf(config.max_position_delta);
        let max_position_delta = config.max_position_delta;
        let x_min = f64::max(MIN_POS, self.x - max_position_delta);
        let x_max = f64::min(config.width, self.x + max_position_delta);

        let y_min = f64::max(MIN_POS, self.y - max_position_delta);
        let y_max = f64::min(config.height, self.y + max_position_delta);
        self.x = random_in_range(x_min, x_max);
        self.y = random_in_range(y_min, y_max);
    }
}

#[derive(Clone, Debug, Copy)]
struct Opacity(f64);

impl Display for Opacity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Opacity {
    fn rand() -> Self {
        Opacity(random())
    }

    fn update(&mut self) -> () {
        self.0 = random()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Hue(f64);

impl Hue {
    pub fn new(config: &CircleConfig) -> Self {
        Hue(random_in_range(config.color_min, config.color_max))
    }
    pub fn update(&mut self, config: &CircleConfig) {
        let max_color_delta = config.max_color_delta as f64;

        // This messes up the circleness that 0 = 360, hmmm
        self.0 =
            saturating_random_in_range(self.0, max_color_delta, config.color_min, config.color_max)
        // self.0 = random_in_range(self.0 - max_color_delta, self.0 + max_color_delta) % 360.0
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Color {
    hue: Hue,
    saturation: f64,
    lightness: f64,
    opacity: Opacity,
}

impl Color {
    pub fn new(config: &CircleConfig) -> Self {
        Color {
            hue: Hue::new(&config),
            saturation: random_in_range(0.5, 1.0),
            lightness: random_in_range(0.25, 0.75),
            opacity: Opacity::rand(),
        }
    }

    pub fn update(&mut self, config: &CircleConfig) {
        self.hue.update(&config);
        self.opacity.update();
        let delta = config.max_color_delta as f64 / 360. * 1.5;
        self.saturation = saturating_random_in_range(self.saturation, delta, 0.4, 1.0);
        self.lightness = saturating_random_in_range(self.lightness, delta, 0.1, 0.9);
    }

    pub fn to_hsl(&self) -> String {
        format!(
            "hsl({:.3}, {:.4}%, {:.4}%)",
            self.hue.0,
            self.saturation * 100.,
            self.lightness * 100.0
        )
    }

    pub fn to_hsla(&self) -> String {
        format!(
            "hsl({:.3}, {:.3}%, {:.3}%, {:3})",
            self.hue.0,
            self.saturation * 100.,
            self.lightness * 100.0,
            self.opacity
        )
    }

    pub fn to_slightly_darker_color(self) -> Self {
        Self {
            lightness: f64::max(0.0, self.lightness - 0.1),
            ..self
        }
    }

    pub fn to_string(&self) -> String {
        self.to_hsla()
    }
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub position: Position,
    pub color: Color,
    pub radius: f64,
}

impl Circle {
    pub fn new(config: &Config, circle_config: &CircleConfig) -> Self {
        Circle {
            position: Position::new(&circle_config),
            color: Color::new(&circle_config),
            radius: config.radius,
        }
    }

    pub fn update(&mut self, config: &CircleConfig) {
        self.position.update(&config, self.radius);
        self.color.update(&config);
    }

    pub fn color(&self) -> String {
        self.color.to_string()
    }
}

#[derive(Clone)]
pub struct Universe {
    pub config: Config,
    pub circle_config: CircleConfig,
    pub circles: Vec<Circle>,
    pub apples: Vec<Apple>,
}

impl Universe {
    pub fn tick(&mut self) {
        self.apples.drain_filter(|apple| Apple::update(apple));

        match self.config.status {
            Status::RUNNING => {
                for circle in self.circles.iter_mut() {
                    circle.update(&self.circle_config)
                }
            }

            Status::PAUSED => {}
        }
    }

    pub fn is_empty(&self) -> bool {
        self.circles.is_empty() && self.apples.is_empty()
    }
    pub fn add_circle(&mut self) {
        self.circles
            .push(Circle::new(&self.config, &self.circle_config))
    }

    pub fn add_apple(&mut self) {
        self.apples.push(Apple {
            circle: Circle::new(&self.config, &self.circle_config),
            config: self.circle_config.clone(),
            steps: self.config.apple_steps,
        })
    }

    fn remaining_apple_steps(&self) -> u32 {
        self.apples
            .iter()
            .max_by(|apple1, apple2| apple1.steps.cmp(&apple2.steps))
            .map_or(0, |apple| apple.steps)
    }

    pub fn steps(&self) -> u32 {
        u32::max(
            u32::min(5000, self.remaining_apple_steps()),
            f64::ceil(self.config.speed.steps() as f64 / (f64::max(1.0, self.circles.len() as f64)))
                as u32,
        )
    }
}

#[derive(Clone)]
pub struct Apple {
    pub circle: Circle,
    pub config: CircleConfig,
    pub steps: u32,
}

impl Apple {
    pub fn update(&mut self) -> bool {
        self.circle.update(&self.config);
        self.steps -= 1;
        self.steps == 0
    }
}

#[derive(Debug, Clone)]
pub struct CircleConfig {
    pub width: f64,
    pub height: f64,
    pub max_position_delta: f64,
    pub max_color_delta: f64,
    pub color_min: f64,
    pub color_max: f64,
    // pub radius: f64,
}

#[derive(Clone)]
pub struct Config {
    pub status: Status,
    pub speed: Speed,
    pub radius: f64,
    pub apple_steps: u32,
    pub bug_checkbox: bool,
    pub color_mode: ColorMode,
    pub initial_height: f64,
    pub initial_width: f64,
}

#[derive(Clone, Copy)]
pub enum Status {
    RUNNING,
    PAUSED,
}

impl Status {
    pub fn toggle(&mut self) {
        *self = match self {
            Status::RUNNING => Status::PAUSED,
            Status::PAUSED => Status::RUNNING,
        }
    }

    // button should display what pressing would do, that is the opposite of current state
    pub fn to_button_display(&self) -> String {
        String::from(match self {
            Status::RUNNING => "⏸",
            Status::PAUSED => "▶️",
        })
    }
}

#[derive(Copy, Clone)]
pub enum Speed {
    NORMAL,
    FAST,
}

impl Speed {
    fn steps(self) -> u32 {
        match self {
            Speed::NORMAL => 1,
            Speed::FAST => 3000,
        }
    }

    pub fn next(self) -> Speed {
        match self {
            Speed::NORMAL => Speed::FAST,
            Speed::FAST => Speed::NORMAL,
        }
    }
    pub fn toggle(&mut self) {
        *self = self.next()
    }

    fn display(self) -> String {
        String::from(match self {
            Speed::NORMAL => "🐢",
            Speed::FAST => "🐇",
        })
    }

    pub fn to_button_display(self) -> String {
        self.next().display()
    }
}

#[derive(Copy, Clone)]
pub enum ColorMode {
    RGB,
    HSL,
}

impl ColorMode {
    pub fn next(self) -> ColorMode {
        match self {
            ColorMode::RGB => ColorMode::HSL,
            ColorMode::HSL => ColorMode::RGB,
        }
    }
    pub fn toggle(&mut self) {
        *self = self.next()
    }

    fn display(self) -> String {
        String::from(match self {
            ColorMode::RGB => "R",
            ColorMode::HSL => "H",
        })
    }

    pub fn to_button_display(self) -> String {
        self.next().display()
    }
}
