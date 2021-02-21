use js_sys::Math::random as js_math_random;
use std::f64;
use std::fmt::{self, Display};

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

const MIN_POS: f64 = 0.0;
const MAX_X_POS: f64 = 500.0;
const MAX_Y_POS: f64 = 250.0;
const RADIUS: f64 = 2.2;
impl Position {
    fn rand() -> Self {
        Position {
            x: random_in_range(MIN_POS, MAX_X_POS),
            y: random_in_range(MIN_POS, MAX_Y_POS),
        }
    }

    fn validate(&self) -> bool {
        self.x >= MIN_POS && self.x <= MAX_X_POS && self.y >= MIN_POS && self.y <= MAX_Y_POS
    }

    fn update(&mut self, max_position_delta: f64) {
        let x_min = f64::max(MIN_POS, self.x - max_position_delta);
        let x_max = f64::min(MAX_X_POS, self.x + max_position_delta);

        let y_min = f64::max(MIN_POS, self.y - max_position_delta);
        let y_max = f64::min(MAX_Y_POS, self.y + max_position_delta);
        self.x = random_in_range(x_min, x_max);
        self.y = random_in_range(y_min, y_max);
        assert!(self.validate());
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

    fn update(&mut self, color_delta: u8) -> () {
        let min = self.bit.saturating_sub(color_delta);
        let max = self.bit.saturating_add(color_delta);
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
    fn rand() -> Self {
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

    fn update(&mut self, color_delta: u8) {
        let update_with_delta = move |x| ColorBit::update(x, color_delta);

        update_with_delta(&mut self.r);
        update_with_delta(&mut self.g);
        update_with_delta(&mut self.b);
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
    pub fn new() -> Self {
        Circle {
            position: Position::rand(),
            color: Color::rand(),
            radius: RADIUS,
        }
    }

    pub fn update(&mut self, position_delta: f64, color_delta: u8) {
        self.position.update(position_delta);
        self.color.update(color_delta);
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
    pub width: f64,
    pub height: f64,
    pub max_position_delta: f64,
    pub max_color_delta: u8,
    pub circles: Vec<Circle>,
}

impl Universe {
    pub fn tick(&mut self) {
        for circle in self.circles.iter_mut() {
            circle.update(self.max_position_delta, self.max_color_delta)
        }
    }
}
