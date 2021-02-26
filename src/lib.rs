use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod circle;
use circle::{Circle, Config, Status, Universe};

fn draw_circle(context: &web_sys::CanvasRenderingContext2d, circle: &Circle, highlight: bool) {
    let color = JsValue::from_str(&circle.color());
    context.set_fill_style(&color);
    context.begin_path();
    if highlight {
        context.set_stroke_style(&JsValue::from_str("rgb(0,0,0)"));
    } else {
        context.set_stroke_style(&color);
    }

    context
        .arc(
            circle.position.x,
            circle.position.y,
            circle.radius,
            0.0,
            f64::consts::PI * 2.0,
        )
        .unwrap();

    context.fill();
    context.stroke();
}

pub fn render(universe: &Universe, canvas: &web_sys::HtmlCanvasElement, highlight: bool) {
    let context = context(&canvas);
    for circle in universe.circles.iter() {
        draw_circle(&context, &circle, highlight)
    }
}

pub fn render_with_highlighting(universe: &Universe) {
    let overlay_canvas = overlay_canvas();
    let default_canvas = default_canvas();
    clear_canvas(&overlay_canvas);
    render(&universe, &default_canvas, false);
    match &universe.config.status {
        Status::RUNNING => {
            render(&universe, &overlay_canvas, true);
        }
        Status::PAUSED => {}
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

fn body() -> web_sys::HtmlElement {
    document().body().unwrap()
}
fn get_canvas_by_id(id: &str) -> web_sys::HtmlCanvasElement {
    document()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap()
}

fn default_canvas() -> web_sys::HtmlCanvasElement {
    get_canvas_by_id("canvas")
}

fn overlay_canvas() -> web_sys::HtmlCanvasElement {
    get_canvas_by_id("overlay-canvas")
}

fn all_canvases() -> Vec<web_sys::HtmlCanvasElement> {
    vec![default_canvas(), overlay_canvas()]
}

fn context(canvas: &web_sys::HtmlCanvasElement) -> web_sys::CanvasRenderingContext2d {
    canvas
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

fn clear_canvas(canvas: &web_sys::HtmlCanvasElement) {
    let context = context(&canvas);

    context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
}
fn clear_board() {
    web_sys::console::log(&js_sys::Array::from(&JsValue::from_str("CLEAR")));
    for canvas in all_canvases() {
        clear_canvas(&canvas)
    }
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let width = body().client_width();
    let height = body().client_height();

    for canvas in all_canvases() {
        canvas.set_height(height as u32);
        canvas.set_width(width as u32);
    }

    let universe = Arc::new(Mutex::new(Universe {
        config: Config {
            status: Status::RUNNING,
            height: height as f64,
            width: width as f64,
            radius: 2.2,
            max_position_delta: 2.3,
            max_color_delta: 5,
        },
        circles: vec![],
    }));

    let distance_slider_div = document()
        .create_element("div")?
        .dyn_into::<web_sys::HtmlDivElement>()?;
    distance_slider_div.set_class_name("control");

    let distance_slider_id = "distance-slider";
    let distance_slider = document()
        .create_element("input")?
        .dyn_into::<web_sys::HtmlInputElement>()?;
    distance_slider.set_class_name("slider");
    distance_slider.set_id(distance_slider_id);
    distance_slider.set_type("range");
    distance_slider.set_min("0");
    distance_slider.set_value("2.3");
    distance_slider.set_max("20");
    distance_slider.set_step("0.1");

    let distance_slider_universe = Arc::clone(&universe);
    let distance_slider_on_change_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You updated a slider!",
        )));

        distance_slider_universe
            .lock()
            .unwrap()
            .config
            .max_position_delta = document()
            .get_element_by_id(distance_slider_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .value_as_number()
    }) as Box<dyn FnMut()>);

    distance_slider.set_onchange(Some(
        distance_slider_on_change_handler.as_ref().unchecked_ref(),
    ));
    distance_slider_on_change_handler.forget();

    distance_slider_div.append_child(&distance_slider)?;

    let color_slider_id = "color-slider";
    let color_slider = document()
        .create_element("input")?
        .dyn_into::<web_sys::HtmlInputElement>()?;
    color_slider.set_class_name("slider");
    color_slider.set_id(color_slider_id);
    color_slider.set_type("range");
    color_slider.set_min("0");
    color_slider.set_value("5");
    color_slider.set_max("50");
    color_slider.set_step("1");

    let color_slider_universe = Arc::clone(&universe);
    let color_slider_on_change_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You updated a slider!",
        )));

        color_slider_universe.lock().unwrap().config.max_color_delta = document()
            .get_element_by_id(color_slider_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .value_as_number()
            as u8
    }) as Box<dyn FnMut()>);

    color_slider.set_onchange(Some(
        color_slider_on_change_handler.as_ref().unchecked_ref(),
    ));
    color_slider_on_change_handler.forget();
    let radius_slider_id = "radius-slider";
    let radius_slider = document()
        .create_element("input")?
        .dyn_into::<web_sys::HtmlInputElement>()?;
    radius_slider.set_class_name("slider");
    radius_slider.set_id(radius_slider_id);
    radius_slider.set_type("range");
    radius_slider.set_min("1");
    radius_slider.set_value("2");
    radius_slider.set_max("20");
    radius_slider.set_step("1");

    let radius_slider_universe = Arc::clone(&universe);
    let radius_slider_on_change_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You updated a slider!",
        )));

        radius_slider_universe.lock().unwrap().config.radius = document()
            .get_element_by_id(radius_slider_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .value_as_number()
    }) as Box<dyn FnMut()>);

    radius_slider.set_onchange(Some(
        radius_slider_on_change_handler.as_ref().unchecked_ref(),
    ));
    radius_slider_on_change_handler.forget();

    let add_button = document()
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .unwrap();

    add_button.set_id("add-button");
    add_button.set_inner_text("+");

    let add_button_universe = Arc::clone(&universe);
    let add_button_on_click_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You pushed a button!",
        )));
        add_button_universe.lock().unwrap().add_circle();
    }) as Box<dyn FnMut()>);

    add_button.set_onclick(Some(add_button_on_click_handler.as_ref().unchecked_ref()));
    add_button_on_click_handler.forget();

    let start_stop_button = document()
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .unwrap();

    let start_stop_button_id = "start-stop-button";
    start_stop_button.set_id(start_stop_button_id);
    start_stop_button.set_inner_text(&universe.lock().unwrap().config.status.to_button_display());

    let universe_clone_2 = Arc::clone(&universe);
    let start_stop_button_on_click_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You pushed the start stop button!",
        )));
        // implementation version 1 of toggling status

        universe_clone_2.lock().unwrap().config.status.toggle();
        let button = document()
            .get_element_by_id(start_stop_button_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlButtonElement>()
            .unwrap();

        button.set_inner_text(
            &universe_clone_2
                .lock()
                .unwrap()
                .config
                .status
                .to_button_display(),
        )

        // implementation version 2 of toggling status
        // let mut local_universe = universe_clone_2.lock().unwrap();
        // local_universe.config.status = match local_universe.config.status {
        //     Status::RUNNING => Status::PAUSED,
        //     Status::PAUSED => Status::RUNNING,
        // }
    }) as Box<dyn FnMut()>);

    start_stop_button.set_onclick(Some(
        start_stop_button_on_click_handler.as_ref().unchecked_ref(),
    ));
    start_stop_button_on_click_handler.forget();

    let trash_button = document()
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .unwrap();

    trash_button.set_id("trash-button");
    trash_button.set_inner_text("🗑️");
    let trash_universe = Arc::clone(&universe);
    let trash_onclick_handler = Closure::wrap(Box::new(move || {
        trash_universe.lock().unwrap().circles.clear();
        clear_board();
    }) as Box<dyn FnMut()>);
    trash_button.set_onclick(Some(trash_onclick_handler.as_ref().unchecked_ref()));
    trash_onclick_handler.forget();

    let br = document()
        .create_element("br")
        .unwrap()
        .dyn_into::<web_sys::HtmlBrElement>()
        .unwrap();

    body().append_child(&start_stop_button)?;
    body().append_child(&add_button)?;
    body().append_child(&trash_button)?;
    body().append_child(&br)?;
    body().append_child(&distance_slider_div)?;
    body().append_child(&color_slider)?;
    body().append_child(&radius_slider)?;

    universe.lock().unwrap().add_circle();

    let main_loop = Rc::new(RefCell::new(None));
    let main_loop_copy = main_loop.clone();

    *main_loop_copy.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let bug_checkbox_value = document()
            .get_element_by_id("bug-checkbox")
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .checked();

        if bug_checkbox_value {
            render(&universe.lock().unwrap(), &default_canvas(), false);
            universe.lock().unwrap().tick();
            render(&universe.lock().unwrap(), &default_canvas(), true);
        } else {
            universe.lock().unwrap().tick();
            render_with_highlighting(&universe.lock().unwrap());
        }

        request_animation_frame(main_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(main_loop_copy.borrow().as_ref().unwrap());
    Ok(())
}
