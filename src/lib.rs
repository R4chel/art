use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod circle;
use circle::{Circle, Config, Status, Universe};

pub enum StrokeColor {
    BLACK,
    FILLCOLOR,
    DARKER,
}

fn draw_circle(
    context: &web_sys::CanvasRenderingContext2d,
    circle: &Circle,
    stroke_color: &StrokeColor,
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

pub fn render(
    universe: &Universe,
    canvas: &web_sys::HtmlCanvasElement,
    stroke_color: &StrokeColor,
) {
    let context = context(&canvas);
    for circle in universe.circles.iter() {
        draw_circle(&context, &circle, &stroke_color)
    }
}

pub fn render_with_highlighting(universe: &Universe) {
    let overlay_canvas = overlay_canvas();
    let default_canvas = default_canvas();
    clear_canvas(&overlay_canvas);
    render(&universe, &default_canvas, &StrokeColor::FILLCOLOR);
    match &universe.config.status {
        Status::RUNNING => {
            render(&universe, &overlay_canvas, &StrokeColor::DARKER);
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
    initial_value: f64,
    step: f64,
}

impl SliderConfig {
    fn create_slider(config: &Self) -> web_sys::HtmlInputElement {
        let slider = document()
            .create_element("input")
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();

        slider.set_class_name("slider");
        slider.set_id(&config.id);
        slider.set_type("range");
        slider.set_min(&config.min.to_string());
        slider.set_value(&config.initial_value.to_string());
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

fn create_br() -> web_sys::HtmlBrElement {
    document()
        .create_element("br")
        .unwrap()
        .dyn_into::<web_sys::HtmlBrElement>()
        .unwrap()
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
            radius: 10.,
            max_position_delta: 6.3,
            max_color_delta: 5,
        },
        circles: vec![],
    }));

    let distance_slider_div = document()
        .create_element("div")?
        .dyn_into::<web_sys::HtmlDivElement>()?;
    distance_slider_div.set_class_name("control");

    let distance_slider_config = SliderConfig {
        id: String::from("distance-slider"),
        title: String::from("Movement Speed"),
        min: 0.0,
        initial_value: universe.lock().unwrap().config.max_position_delta,
        max: 100.0,
        step: 0.1,
    };

    let distance_slider = SliderConfig::create_slider(&distance_slider_config);

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

    distance_slider_div.append_child(&distance_slider)?;

    let color_slider_config = SliderConfig {
        id: String::from("color-slider"),
        title: String::from("Color Speed"),
        min: 0.0,
        initial_value: universe.lock().unwrap().config.max_color_delta as f64,
        max: 50.0,
        step: 1.0,
    };

    let color_slider = SliderConfig::create_slider(&color_slider_config);

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
    let color_slider_div = document()
        .create_element("div")?
        .dyn_into::<web_sys::HtmlDivElement>()?;
    color_slider_div.set_class_name("control");
    color_slider_div.append_child(&color_slider)?;

    let radius_slider_config = SliderConfig {
        id: String::from("radius-slider"),
        title: String::from("Size"),
        min: 1.0,
        initial_value: universe.lock().unwrap().config.radius,
        max: 100.0,
        step: 1.0,
    };

    let radius_slider = SliderConfig::create_slider(&radius_slider_config);

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

    let freeze_button = document()
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .unwrap();

    freeze_button.set_id("freeze-button");
    freeze_button.set_inner_text("🧊");

    let freeze_button_universe = Arc::clone(&universe);
    let freeze_button_on_click_handler = Closure::wrap(Box::new(move || {
        web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
            "You pushed a button!",
        )));
        freeze_button_universe.lock().unwrap().circles.clear();
    }) as Box<dyn FnMut()>);

    freeze_button.set_onclick(Some(
        freeze_button_on_click_handler.as_ref().unchecked_ref(),
    ));
    freeze_button_on_click_handler.forget();

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

    let save_button = document()
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .unwrap();

    save_button.set_id("save-button");
    save_button.set_inner_text("💾");
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

    let new_circle_div = document()
        .create_element("div")?
        .dyn_into::<web_sys::HtmlDivElement>()?;
    new_circle_div.set_class_name("control");
    new_circle_div.append_child(&radius_slider)?;
    new_circle_div.append_child(&add_button)?;
    body().append_child(&start_stop_button)?;
    body().append_child(&freeze_button)?;
    body().append_child(&save_button)?;
    body().append_child(&trash_button)?;

    body().append_child(&new_circle_div)?;
    body().append_child(&distance_slider_div)?;
    body().append_child(&color_slider_div)?;

    universe.lock().unwrap().add_circle();
    universe.lock().unwrap().add_circle();

    let main_loop = Rc::new(RefCell::new(None));
    let main_loop_copy = main_loop.clone();

    clear_board();
    *main_loop_copy.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let bug_checkbox_value = document()
            .get_element_by_id("bug-checkbox")
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap()
            .checked();

        if bug_checkbox_value {
            render(
                &universe.lock().unwrap(),
                &default_canvas(),
                &StrokeColor::FILLCOLOR,
            );
            universe.lock().unwrap().tick();
            render(
                &universe.lock().unwrap(),
                &default_canvas(),
                &StrokeColor::BLACK,
            );
        } else {
            universe.lock().unwrap().tick();
            render_with_highlighting(&universe.lock().unwrap());
        }

        request_animation_frame(main_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(main_loop_copy.borrow().as_ref().unwrap());
    Ok(())
}
