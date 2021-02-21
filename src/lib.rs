use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod circle;
use circle::{Circle, Universe};

fn draw_circle(context: &web_sys::CanvasRenderingContext2d, circle: &Circle) {
    let color = JsValue::from_str(&circle.color());
    context.begin_path();
    context.set_fill_style(&color);
    context.set_stroke_style(&color);

    context
        .arc(
            circle.x_position(),
            circle.y_position(),
            circle.radius(),
            0.0,
            f64::consts::PI * 2.0,
        )
        .unwrap();

    context.fill();
    context.stroke();
}
pub fn render(universe: &Universe) {
    let context = context();
    for circle in universe.circles.iter() {
        draw_circle(&context, &circle)
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn canvas() -> web_sys::HtmlCanvasElement {
    document()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap()
}

fn context() -> web_sys::CanvasRenderingContext2d {
    canvas()
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust!");

    body.append_child(&val)?;

    let mut universe = Universe {
        height: 250.0,
        width: 500.0,
        max_position_delta: 2.3,
        max_color_delta: 5,
        circles: vec![Circle::new()],
    };

    render(&universe);

    for _ in 0..1000 {
        universe.tick();
        render(&universe);
    }

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
