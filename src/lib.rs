use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod circle;
use circle::{Circle, Config, Speed, Status, Universe};

#[derive(Copy, Clone)]
pub enum StrokeColor {
    BLACK,
    FILLCOLOR,
    DARKER,
}

fn draw_circle(
    context: &web_sys::CanvasRenderingContext2d,
    circle: &Circle,
    stroke_color: StrokeColor,
) {
    let color = JsValue::from_str(&circle.color());
    context.set_fill_style(&color);
    context.begin_path();
    let stroke_style = match stroke_color {
        StrokeColor::BLACK => JsValue::from_str("rgb(0,0,0)"),
        StrokeColor::FILLCOLOR => color,
        StrokeColor::DARKER => JsValue::from_str(&circle.color.to_slightly_darker_color()),
    };
    context.set_stroke_style(&stroke_style);

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

pub fn render(universe: &Universe, canvas: &web_sys::HtmlCanvasElement, stroke_color: StrokeColor) {
    let context = context(&canvas);
    for circle in universe.circles.iter() {
        draw_circle(&context, &circle, stroke_color)
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

fn blank_canvas(canvas: &web_sys::HtmlCanvasElement) {
    let context = context(&canvas);

    context.set_fill_style(&JsValue::from_str("white"));
    context.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
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
    blank_canvas(&default_canvas())
}

struct SliderConfig {
    title: String,
    id: String,
    min: f64,
    max: f64,
    step: f64,
    of_universe: fn(&Universe) -> f64,
}

impl SliderConfig {
    fn create_slider(config: &Self, universe: &Universe) -> web_sys::HtmlInputElement {
        let slider = document()
            .create_element("input")
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();

        slider.set_class_name("slider");
        slider.set_id(&config.id);
        slider.set_type("range");
        slider.set_min(&config.min.to_string());
        slider.set_value(&(config.of_universe)(universe).to_string());
        slider.set_max(&config.max.to_string());
        slider.set_step(&config.step.to_string());
        slider.set_title(&config.title);
        slider
    }

    fn get_value(&self) -> f64 {
        document()
            .get_element_by_id(&self.id)
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .value_as_number()
    }
}
fn label(id: &str, text: &str) -> web_sys::HtmlLabelElement {
    let label = document()
        .create_element("label")
        .unwrap()
        .dyn_into::<web_sys::HtmlLabelElement>()
        .unwrap();
    label.set_html_for(id);
    label.set_inner_text(&text);
    label
}

fn control_div(
    input: &web_sys::HtmlInputElement,
    id: &str,
    left_label: Option<&str>,
) -> web_sys::HtmlDivElement {
    let div = document()
        .create_element("div")
        .unwrap()
        .dyn_into::<web_sys::HtmlDivElement>()
        .unwrap();
    div.set_class_name("control");
    match left_label {
        None => {}
        Some(text) => {
            let label = label(id, text);
            div.append_child(&label).unwrap();
        }
    }
    div.append_child(&input).unwrap();

    div
}

pub fn new_button(id: &str, text: &str) -> web_sys::HtmlButtonElement {
    let button = document()
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .unwrap();

    button.set_id(id);
    button.set_inner_text(text);
    button
}

fn new_checkbox(id: &str, text: &str) -> web_sys::HtmlDivElement {
    let checkbox = document()
        .create_element("input")
        .unwrap()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap();

    checkbox.set_id(&id);
    checkbox.set_type("checkbox");

    control_div(&checkbox, id, Some(&text))
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
            speed: Speed::NORMAL,
            height: height as f64,
            width: width as f64,
            radius: 10.,
            max_position_delta: 6.3,
            max_color_delta: 5,
        },
        circles: vec![],
    }));

    let distance_slider_id = "distance-slider";
    let distance_slider_config = SliderConfig {
        id: String::from(distance_slider_id),
        title: String::from("Movement Speed"),
        min: 0.0,
        max: 100.0,
        step: 0.01,
        of_universe: (move |universe| universe.config.max_position_delta),
    };

    let distance_slider =
        SliderConfig::create_slider(&distance_slider_config, &universe.lock().unwrap());

    let distance_slider_universe = Arc::clone(&universe);
    let distance_slider_on_change_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You updated a slider!",
        )));

        distance_slider_universe
            .lock()
            .unwrap()
            .config
            .max_position_delta = SliderConfig::get_value(&distance_slider_config)
    }) as Box<dyn FnMut()>);

    distance_slider.set_oninput(Some(
        distance_slider_on_change_handler.as_ref().unchecked_ref(),
    ));

    distance_slider_on_change_handler.forget();

    let color_slider_id = "color-slider";
    let color_slider_config = SliderConfig {
        id: String::from(color_slider_id),
        title: String::from("Color Speed"),
        min: 0.0,
        max: 50.0,
        step: 1.0,
        of_universe: (move |universe| universe.config.max_color_delta as f64),
    };

    let color_slider = SliderConfig::create_slider(&color_slider_config, &universe.lock().unwrap());

    let color_slider_universe = Arc::clone(&universe);
    let color_slider_on_change_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You updated a slider!",
        )));

        color_slider_universe.lock().unwrap().config.max_color_delta =
            SliderConfig::get_value(&color_slider_config) as u8
    }) as Box<dyn FnMut()>);

    color_slider.set_oninput(Some(
        color_slider_on_change_handler.as_ref().unchecked_ref(),
    ));

    color_slider_on_change_handler.forget();

    let radius_slider_id = "radius-slider";
    let radius_slider_config = SliderConfig {
        id: String::from(radius_slider_id),
        title: String::from("Size"),
        min: 1.0,
        max: 100.0,
        step: 1.0,
        of_universe: (move |universe| universe.config.radius),
    };

    let radius_slider =
        SliderConfig::create_slider(&radius_slider_config, &universe.lock().unwrap());

    let radius_slider_universe = Arc::clone(&universe);
    let radius_slider_on_change_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You updated a slider!",
        )));

        radius_slider_universe.lock().unwrap().config.radius =
            SliderConfig::get_value(&radius_slider_config)
    }) as Box<dyn FnMut()>);

    radius_slider.set_oninput(Some(
        radius_slider_on_change_handler.as_ref().unchecked_ref(),
    ));
    radius_slider_on_change_handler.forget();

    let add_button_id = "add-button";
    let add_button = new_button(add_button_id, "+");
    let add_button_universe = Arc::clone(&universe);
    let add_button_on_click_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You pushed a button!",
        )));
        add_button_universe.lock().unwrap().add_circle();

        document()
            .get_element_by_id(add_button_id)
            .unwrap()
            .set_class_name("");
    }) as Box<dyn FnMut()>);

    add_button.set_onclick(Some(add_button_on_click_handler.as_ref().unchecked_ref()));
    add_button_on_click_handler.forget();

    let freeze_button = new_button("freeze-button", "🧊");
    let freeze_button_universe = Arc::clone(&universe);
    let freeze_button_on_click_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You pushed a button!",
        )));
        freeze_button_universe.lock().unwrap().circles.clear();
        document()
            .get_element_by_id(add_button_id)
            .unwrap()
            .set_class_name("highlight");
    }) as Box<dyn FnMut()>);

    freeze_button.set_onclick(Some(
        freeze_button_on_click_handler.as_ref().unchecked_ref(),
    ));
    freeze_button_on_click_handler.forget();

    let start_stop_button_id = "start-stop-button";
    let start_stop_button = new_button(
        start_stop_button_id,
        &universe.lock().unwrap().config.status.to_button_display(),
    );
    let start_stop_universe = Arc::clone(&universe);
    let start_stop_button_on_click_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You pushed the start stop button!",
        )));
        // implementation version 1 of toggling status
        start_stop_universe.lock().unwrap().config.status.toggle();

        // implementation version 2 of toggling status
        // let mut local_universe = universe_clone_2.lock().unwrap();
        // local_universe.config.status = match local_universe.config.status {
        //     Status::RUNNING => Status::PAUSED,
        //     Status::PAUSED => Status::RUNNING,
        // }

        let button = document()
            .get_element_by_id(start_stop_button_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlButtonElement>()
            .unwrap();

        button.set_inner_text(
            &start_stop_universe
                .lock()
                .unwrap()
                .config
                .status
                .to_button_display(),
        )
    }) as Box<dyn FnMut()>);

    start_stop_button.set_onclick(Some(
        start_stop_button_on_click_handler.as_ref().unchecked_ref(),
    ));
    start_stop_button_on_click_handler.forget();

    let speed_button_id = "speed-button";
    let speed_button = new_button(
        speed_button_id,
        &universe.lock().unwrap().config.speed.to_button_display(),
    );

    let speed_universe = Arc::clone(&universe);
    let speed_button_on_click_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You pushed the speed button!",
        )));

        speed_universe.lock().unwrap().config.speed.toggle();

        let button = document()
            .get_element_by_id(speed_button_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlButtonElement>()
            .unwrap();

        button.set_inner_text(
            &speed_universe
                .lock()
                .unwrap()
                .config
                .speed
                .to_button_display(),
        )
    }) as Box<dyn FnMut()>);

    speed_button.set_onclick(Some(speed_button_on_click_handler.as_ref().unchecked_ref()));
    speed_button_on_click_handler.forget();

    let trash_button = new_button("trash-button", "🗑️");
    let trash_universe = Arc::clone(&universe);
    let trash_onclick_handler = Closure::wrap(Box::new(move || {
        trash_universe.lock().unwrap().circles.clear();
        clear_board();
        document()
            .get_element_by_id(add_button_id)
            .unwrap()
            .set_class_name("highlight");
    }) as Box<dyn FnMut()>);
    trash_button.set_onclick(Some(trash_onclick_handler.as_ref().unchecked_ref()));
    trash_onclick_handler.forget();

    let save_button = new_button("save-button", "💾");
    let save_onclick_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(&format!(
            "you tried saving!"
        ))));

        let image = default_canvas().to_data_url().unwrap();

        let anchor = document()
            .create_element("a")
            .unwrap()
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .unwrap();

        anchor.set_href(&image);
        anchor.set_download("art.png");
        anchor.click();
    }) as Box<dyn FnMut()>);
    save_button.set_onclick(Some(save_onclick_handler.as_ref().unchecked_ref()));
    save_onclick_handler.forget();

    let bug_checkbox_id = "bug-checkbox";
    let bug_checkbox = new_checkbox(bug_checkbox_id, "🐛");

    let new_circle_div = control_div(&radius_slider, &radius_slider_id, None);
    new_circle_div.append_child(&add_button)?;

    let distance_slider_div = control_div(&distance_slider, &distance_slider_id, Some("↔"));

    body().append_child(&start_stop_button)?;
    body().append_child(&speed_button)?;

    body().append_child(&freeze_button)?;
    body().append_child(&save_button)?;
    body().append_child(&trash_button)?;

    body().append_child(&new_circle_div)?;
    body().append_child(&bug_checkbox)?;

    body().append_child(&distance_slider_div)?;
    body().append_child(&(control_div(&color_slider, &color_slider_id, Some("🌈"))))?;

    universe.lock().unwrap().add_circle();
    universe.lock().unwrap().add_circle();

    let main_loop = Rc::new(RefCell::new(None));
    let main_loop_copy = main_loop.clone();

    clear_board();
    *main_loop_copy.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let steps = universe.lock().unwrap().steps();

        let overlay_canvas = overlay_canvas();
        let default_canvas = default_canvas();

        let bug_checkbox_value = document()
            .get_element_by_id(bug_checkbox_id)
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .checked();

        let mut universe = universe.lock().unwrap();
        for _ in 0..steps {
            if bug_checkbox_value {
                render(&universe, &default_canvas, StrokeColor::FILLCOLOR);
                universe.tick();
                render(&universe, &default_canvas, StrokeColor::BLACK);
            } else {
                universe.tick();
                render(&universe, &default_canvas, StrokeColor::FILLCOLOR);
            }
        }

        clear_canvas(&overlay_canvas);
        match &universe.config.status {
            Status::RUNNING => {
                render(&universe, &overlay_canvas, StrokeColor::DARKER);
            }
            Status::PAUSED => {}
        }

        request_animation_frame(main_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(main_loop_copy.borrow().as_ref().unwrap());
    Ok(())
}
