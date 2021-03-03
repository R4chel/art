use hsl::HSL;
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

impl Position {
    fn new(config: &Config) -> Self {
        Position {
            x: random_in_range(MIN_POS, config.width),
            y: random_in_range(MIN_POS, config.height),
        }
    }

    fn update(&mut self, config: &Config, _radius: f64) {
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

#[derive(Clone, Debug)]
struct ColorBit(u8);

impl Display for ColorBit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ColorBit {
    fn rand() -> Self {
        ColorBit(f64::round(random() * 255 as f64) as u8)
    }

    fn update(&mut self, config: &Config) -> () {
        let min = self.0.saturating_sub(config.max_color_delta);
        let max = self.0.saturating_add(config.max_color_delta);
        self.0 = f64::floor(random() * (max - min + 1) as f64) as u8 + min;
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct Color {
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
        format!("rgb({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }

    fn update(&mut self, config: &Config) {
        self.r.update(&config);
        self.g.update(&config);
        self.b.update(&config);
        self.a.update();
    }

    pub fn to_slightly_darker_color(&self) -> String {
        let mut hsl = HSL::from_rgb(&[self.r.0, self.g.0, self.b.0]);
        hsl.l = f64::max(0.0, hsl.l - 0.1);
        let (r, g, b) = hsl.to_rgb();
        format!("rgb({}, {}, {})", r, g, b)
    }
}

#[derive(Debug, Clone)]
pub struct Circle {
    pub position: Position,
    pub color: Color,
    pub radius: f64,
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
        self.position.update(&config, self.radius);
        self.color.update(&config);
    }

    pub fn color(&self) -> String {
        self.color.to_rgba()
    }
}

#[derive(Clone)]
pub struct Universe {
    pub config: Config,
    pub circles: Vec<Circle>,
}

impl Universe {
    pub fn tick(&mut self) {
        match self.config.status {
            Status::RUNNING => {
                for circle in self.circles.iter_mut() {
                    circle.update(&self.config)
                }
            }

            Status::PAUSED => {}
        }
    }

    pub fn add_circle(&mut self) {
        self.circles.push(Circle::new(&self.config))
    }

    pub fn steps(&self) -> u8 {
        f64::ceil(self.config.speed.steps() as f64 / self.circles.len() as f64) as u8
    }
}

#[derive(Clone)]
pub struct Config {
    pub status: Status,
    pub speed: Speed,
    pub width: f64,
    pub height: f64,
    pub max_position_delta: f64,
    pub max_color_delta: u8,
    pub radius: f64,
}

#[derive(Clone)]
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
