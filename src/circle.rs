use js_sys::Math::random as js_math_random;
use std::f64;
use std::fmt::{self, Display};

const MIN_POS: f64 = 0.0;
#[derive(Debug)]
struct Position {
    x: f64,
    y: f64,
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

impl Position {
    fn new(config: &Config) -> Self {
        Position {
            x: random_in_range(MIN_POS, config.width),
            y: random_in_range(MIN_POS, config.height),
        }
    }

    fn update(&mut self, config: &Config) {
        let x_min = f64::max(MIN_POS, self.x - config.max_position_delta);
        let x_max = f64::min(config.width, self.x + config.max_position_delta);

        let y_min = f64::max(MIN_POS, self.y - config.max_position_delta);
        let y_max = f64::min(config.height, self.y + config.max_position_delta);
        self.x = random_in_range(x_min, x_max);
        self.y = random_in_range(y_min, y_max);
    }
}

#[derive(Debug)]
struct ColorBit {
    bit: u8,
}

impl Display for ColorBit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bit)
    }
}

impl ColorBit {
    fn rand() -> Self {
        ColorBit {
            bit: f64::round(random() * 255 as f64) as u8,
        }
    }

    fn update(&mut self, config: &Config) -> () {
        let min = self.bit.saturating_sub(config.max_color_delta);
        let max = self.bit.saturating_add(config.max_color_delta);
        self.bit = f64::floor(random() * (max - min + 1) as f64) as u8 + min;
    }
}

#[derive(Debug)]
struct Opacity {
    bit: f64,
}

impl Display for Opacity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bit)
    }
}

impl Opacity {
    fn rand() -> Self {
        Opacity { bit: random() }
    }

    fn update(&mut self) -> () {
        self.bit = random()
    }
}

#[derive(Debug)]
struct Color {
    r: ColorBit,
    g: ColorBit,
    b: ColorBit,
    a: Opacity,
}

impl Color {
    fn new() -> Self {
        Color {
            r: ColorBit::rand(),
            g: ColorBit::rand(),
            b: ColorBit::rand(),
            a: Opacity::rand(),
        }
    }

    fn to_rgba(&self) -> String {
        format!(
            "rgb({}, {}, {}, {})",
            self.r.bit, self.g.bit, self.b.bit, self.a.bit
        )
    }

    fn update(&mut self, config: &Config) {
        self.r.update(&config);
        self.g.update(&config);
        self.b.update(&config);
        self.a.update();
    }
}

#[derive(Debug)]
pub struct Circle {
    position: Position,
    color: Color,
    radius: f64,
}

impl Circle {
    pub fn new(config: &Config) -> Self {
        Circle {
            position: Position::new(&config),
            color: Color::new(),
            radius: config.radius,
        }
    }

    pub fn update(&mut self, config: &Config) {
        self.position.update(&config);
        self.color.update(&config);
    }

    pub fn color(&self) -> String {
        self.color.to_rgba()
    }

    pub fn x_position(&self) -> f64 {
        self.position.x
    }
    pub fn y_position(&self) -> f64 {
        self.position.y
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }
}

pub struct Universe {
    pub config: Config,
    pub circles: Vec<Circle>,
}

impl Universe {
    pub fn tick(&mut self) {
        for circle in self.circles.iter_mut() {
            circle.update(&self.config)
        }
    }

    pub fn add_circle(&mut self) {
        self.circles.push(Circle::new(&self.config))
    }
}

// const config.height: f64 = 250.0;
// const config.radius: f64 = 2.2;
pub struct Config {
    pub width: f64,
    pub height: f64,
    pub max_position_delta: f64,
    pub max_color_delta: u8,
    pub radius: f64,
}
