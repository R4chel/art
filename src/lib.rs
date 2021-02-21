use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod circle;
use circle::{Circle, Config, Universe};

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

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let mut universe = Universe {
        config: Config {
            height: 250.0,
            width: 500.0,
            radius: 2.2,
            max_position_delta: 2.3,
            max_color_delta: 5,
        },
        circles: vec![],
    };

    universe.add_circle();

    let main_loop = Rc::new(RefCell::new(None));
    let main_loop_copy = main_loop.clone();

    *main_loop_copy.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        universe.tick();
        render(&universe);

        request_animation_frame(main_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(main_loop_copy.borrow().as_ref().unwrap());

    Ok(())
}

#[wasm_bindgen]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}
