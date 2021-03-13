#![feature(drain_filter)]
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod circle;
use circle::{
    Circle, CircleConfig, ColorConfig, ColorMode, ColorParamConfig, Config, Speed, Status, Universe,
};

const ADD_BUTTON_ID: &str = "add-button";
const APPLE_BUTTON_ID: &str = "apple-button";

#[derive(Copy, Clone)]
pub enum StrokeColor {
    BLACK,
    FILLCOLOR,
}

fn circle_to_svg(circle: &Circle) -> web_sys::SvgCircleElement {
    let svg_circle = document()
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "circle")
        .unwrap()
        .dyn_into::<web_sys::SvgCircleElement>()
        .unwrap();
    svg_circle
        .set_attribute("r", &circle.radius.to_string())
        .unwrap();
    svg_circle
        .set_attribute("cx", &circle.position.x.to_string())
        .unwrap();
    svg_circle
        .set_attribute("cy", &circle.position.y.to_string())
        .unwrap();
    svg_circle.set_attribute("fill", &circle.color()).unwrap();
    svg_circle
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

// Long term don't want render to mutate universe. Something to think about
pub fn render_svg(universe: &mut Universe, svg: &web_sys::SvgElement) {
    for circle in universe.circles.iter_mut() {
        if circle.dirty {
            svg.append_child(&circle_to_svg(&circle)).unwrap();
            circle.dirty = false;
        }
    }

    for apple in universe.apples.iter_mut() {
        if apple.circle.dirty {
            svg.append_child(&circle_to_svg(&apple.circle)).unwrap();
            apple.circle.dirty = false;
        }
    }
}

pub fn render(universe: &Universe, canvas: &web_sys::HtmlCanvasElement) {
    let context = context(&canvas);

    for circle in universe.circles.iter() {
        if universe.config.bug_checkbox {
            draw_circle(&context, &circle, StrokeColor::BLACK);
        } else {
        };
        draw_circle(&context, &circle, StrokeColor::FILLCOLOR);
    }

    for apple in universe.apples.iter() {
        if universe.config.bug_checkbox {
            draw_circle(&context, &apple.circle, StrokeColor::BLACK);
        } else {
        };
        draw_circle(&context, &apple.circle, StrokeColor::FILLCOLOR);
    }
}

pub fn highlight(
    universe: &Universe,
    canvas: &web_sys::HtmlCanvasElement,
    stroke_color: StrokeColor,
) {
    let context = context(&canvas);
    for circle in universe.circles.iter() {
        draw_circle(&context, &circle, stroke_color);
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
fn clear_svg(svg: &mut web_sys::SvgElement) {
    while let Some(child) = svg.first_child() {
        svg.remove_child(&child).unwrap();
    }
}

fn clear_board(svg: &mut web_sys::SvgElement) {
    web_sys::console::log(&js_sys::Array::from(&JsValue::from_str("CLEAR")));
    clear_svg(svg);
    for canvas in all_canvases() {
        clear_canvas(&canvas)
    }
    blank_canvas(&default_canvas())
}

#[derive(Clone)]
struct SliderConfig<OfUniverse : FnOnce(&Universe) -> f64  + Clone + 'static>  {
    title: String,
    id: String,
    min: f64,
    max: f64,
    step: f64,
    of_universe: OfUniverse,
    on_update: fn(&mut Universe, f64) -> (),
    left_label: Option<String>,
}

impl<OfUniverse: FnOnce(&Universe) -> f64 + Clone + 'static> SliderConfig<OfUniverse> {
    fn create_slider(&self, universe: &Arc<Mutex<Universe>>) -> web_sys::HtmlDivElement {
        let slider = document()
            .create_element("input")
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();


        let of_universe = self.of_universe.clone();
        let current_value = of_universe(&*universe.lock().unwrap()).to_string();

        slider.set_class_name("slider");
        slider.set_id(&self.id);
        slider.set_name(&self.id);
        slider.set_type("range");
        slider.set_min(&self.min.to_string());
        slider.set_value(&current_value);
        slider.set_max(&self.max.to_string());
        slider.set_step(&self.step.to_string());
        slider.set_title(&self.title);

        let display = document()
            .create_element("input")
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();

        let mut display_id_tmp = String::from(&self.id);
        display_id_tmp.push_str("-input");
        let display_id = display_id_tmp;

        let display_id_clone = display_id.clone();
        display.set_id(&display_id);
        display.set_name(&display_id);

        display.set_type("number");
        display.set_min(&self.min.to_string());
        display.set_value(&current_value);
        display.set_max(&self.max.to_string());
        display.set_step(&self.step.to_string());

        let div = new_control_div();
        match self.left_label {
            None => {}
            Some(ref text) => {
                let label = label(&self.id, &text);
                div.append_child(&label).unwrap();
            }
        }

        div.append_child(&slider).unwrap();
        div.append_child(&display).unwrap();

        let slider_id = String::from(&self.id);
        let self_clone = self.clone();
        let slider_universe = Arc::clone(&universe);
        let slider_on_change_handler = Closure::wrap(Box::new(move || {
            web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(&format!(
                "You updated the {}!",
                &self_clone.id
            ))));

            let value = self_clone.get_value();

            (&self_clone.on_update)(&mut slider_universe.lock().unwrap(), value);

            let display = document()
                .get_element_by_id(&display_id)
                .unwrap()
                .dyn_into::<web_sys::HtmlInputElement>()
                .unwrap();

            display.set_value(&value.to_string());
        }) as Box<dyn FnMut()>);

        slider.set_oninput(Some(slider_on_change_handler.as_ref().unchecked_ref()));

        let display_self_clone = self.clone();

        let display_universe = Arc::clone(&universe);
        let display_on_change_handler = Closure::wrap(Box::new(move || {
            let value = document()
                .get_element_by_id(&display_id_clone)
                .unwrap()
                .dyn_into::<web_sys::HtmlInputElement>()
                .unwrap()
                .value_as_number();

            (&display_self_clone.on_update)(&mut display_universe.lock().unwrap(), value);

            let slider = document()
                .get_element_by_id(&slider_id)
                .unwrap()
                .dyn_into::<web_sys::HtmlInputElement>()
                .unwrap();

            slider.set_value(&value.to_string());
        }) as Box<dyn FnMut()>);

        display.set_oninput(Some(display_on_change_handler.as_ref().unchecked_ref()));

        slider_on_change_handler.forget();
        display_on_change_handler.forget();
        div
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

// pub struct RangeConfig {
//     absolute_max : f64,
//     absolue_min : f64,
//     step : f64,
// }

// pub struct MoreDetailedColorParamConfig {
//     delta : RangeConfig,
//     min : RangeConfig,
//     max : RangeConfig,

// }

// struct ColorParamSubsection {
//     name: String
//         ,
//     emoji: String,
//     of_color_param_config : fn(&ColorParamConfig) -> f64,
//     on_update: fn(&mut ColorParamConfig, f64) -> (),
//     range_config : RangeConfig
// }

// struct ColorParamConfigSliderConfigV2<> {
//     // title: String,
//     id: String,
//     step: f64,
//     of_universe: fn(&Universe) -> ColorParamConfig,
//     config_config : MoreDetailedColorParamConfig,
//     // on_update: fn(&mut Universe, ColorParamConfig) -> (),
//     // left_label: Option<String>,
// }

// impl ColorParamConfigSliderConfigV2 {
    

//     fn one_field_section(&self, subsection_config :ColorParamSubsection,  universe: &Arc<Mutex<Universe>>)-> web_sys::HtmlDivElement {
//         let slider_config = SliderConfig {
//             id: String::from(&format!("{}-{}", self.id, subsection_config.name)),
//             title: String::from(&format!("{} {}", self.id, subsection_config.name)),
//             left_label: Some(subsection_config.emoji),
//             min: subsection_config.range_config.absolue_min, 
//             max: subsection_config.range_config.absolute_max,
//             of_universe: (move |universe| ( subsection_config.of_color_param_config )( &(self.of_universe)(universe))),
//             on_update: (move |_universe, _value| {
                
//                 // (subsection_config.on_update)((self.on_update)universe )
//             }),
//             step: subsection_config.range_config.step,
//         };

//         let delta_slider = SliderConfig::create_slider(&slider_config, &universe);
//         delta_slider   
//     }

//     fn create_config_section(
//         &self,
//         universe: &Arc<Mutex<Universe>>,
//     ) -> web_sys::HtmlDivElement {
//         let div = new_control_div();
//         let initial_config = (self.of_universe)(&universe.lock().unwrap());

//         let delta_slider =
//             self.one_field_section(
//                 ColorParamSubsection {
                    
//                 }
//                 , universe);
//         let min_value_slider_config = SliderConfig {
//             id: String::from(&format!("{}-{}", self.id, "min_value")),
//             left_label: Some(String::from("↓")),
//             min: 0.0,
//             max: initial_config.max_value,
//             of_universe: (move |universe| universe.circle_config.color_config.hue_config.min_value),
//             on_update: (move |universe, value| {
//                 universe.circle_config.color_config.hue_config.min_value = value
//             }),
//             step: self.step,
//             title: String::from("Hue min_value"),
//         };

//         let min_value_slider = SliderConfig::create_slider(&min_value_slider_config, &universe);

//         let max_value_slider_config = SliderConfig {
//             id: String::from(&format!("{}-{}", self.id, "max_value")),
//             left_label: Some(String::from("↑")),
//             min: initial_config.min_value,
//             max: initial_config.max_value,
//             of_universe: (move |universe| universe.circle_config.color_config.hue_config.max_value),
//             on_update: (move |universe, value| {
//                 universe.circle_config.color_config.hue_config.max_value = value
//             }),
//             step: self.step,
//             title: String::from("Hue max_value"),
//         };

//         let max_value_slider = SliderConfig::create_slider(&max_value_slider_config, &universe);

//         div.set_inner_text("Hue");
//         div.append_child(&delta_slider).unwrap();
//         div.append_child(&min_value_slider).unwrap();
//         div.append_child(&max_value_slider).unwrap();
//         div
//     }
// }





#[derive(Clone)]
struct ColorParamConfigSliderConfig {
    // title: String,
    id: String,
    step: f64,
    of_universe: fn(&Universe) -> ColorParamConfig,
    // on_update: fn(&mut Universe, ColorParamConfig) -> (),
    // left_label: Option<String>,
}

impl ColorParamConfigSliderConfig {
    
     
    fn hue_create_section_slider(
        &self,
        universe: &Arc<Mutex<Universe>>,
    ) -> web_sys::HtmlDivElement {
        let div = new_control_div();
        let initial_config = (self.of_universe)(&universe.lock().unwrap());
        let delta_slider_config = SliderConfig {
            id: String::from(&format!("{}-{}", self.id, "delta")),
            left_label: Some(String::from("Δ")),
            min: 0.0,
            max: initial_config.max_value - initial_config.min_value,
            of_universe: (move |universe| universe.circle_config.color_config.hue_config.max_delta),
            on_update: (move |universe, value| {
                universe.circle_config.color_config.hue_config.max_delta = value
            }),
            step: self.step,
            title: String::from("Hue delta"),
        };

        let delta_slider = SliderConfig::create_slider(&delta_slider_config, &universe);

        let min_value_slider_config = SliderConfig {
            id: String::from(&format!("{}-{}", self.id, "min_value")),
            left_label: Some(String::from("↓")),
            min: 0.0,
            max: initial_config.max_value,
            of_universe: (move |universe| universe.circle_config.color_config.hue_config.min_value),
            on_update: (move |universe, value| {
                universe.circle_config.color_config.hue_config.min_value = value
            }),
            step: self.step,
            title: String::from("Hue min_value"),
        };

        let min_value_slider = SliderConfig::create_slider(&min_value_slider_config, &universe);

        let max_value_slider_config = SliderConfig {
            id: String::from(&format!("{}-{}", self.id, "max_value")),
            left_label: Some(String::from("↑")),
            min: initial_config.min_value,
            max: initial_config.max_value,
            of_universe: (move |universe| universe.circle_config.color_config.hue_config.max_value),
            on_update: (move |universe, value| {
                universe.circle_config.color_config.hue_config.max_value = value
            }),
            step: self.step,
            title: String::from("Hue max_value"),
        };

        let max_value_slider = SliderConfig::create_slider(&max_value_slider_config, &universe);

        div.set_inner_text("Hue");
        div.append_child(&delta_slider).unwrap();
        div.append_child(&min_value_slider).unwrap();
        div.append_child(&max_value_slider).unwrap();
        div
    }

    fn saturation_create_section_slider(
        &self,
        universe: &Arc<Mutex<Universe>>,
    ) -> web_sys::HtmlDivElement {
        let div = new_control_div();
        let initial_config = (self.of_universe)(&universe.lock().unwrap());
        let delta_slider_config = SliderConfig {
            id: String::from(&format!("{}-{}", self.id, "delta")),
            left_label: Some(String::from("Δ")),
            min: 0.0,
            max: initial_config.max_value - initial_config.min_value,
            of_universe: (move |universe| {
                universe
                    .circle_config
                    .color_config
                    .saturation_config
                    .max_delta
            }),
            on_update: (move |universe, value| {
                universe
                    .circle_config
                    .color_config
                    .saturation_config
                    .max_delta = value
            }),
            step: self.step,
            title: String::from("Saturation delta"),
        };

        let delta_slider = SliderConfig::create_slider(&delta_slider_config, &universe);

        let min_value_slider_config = SliderConfig {
            id: String::from(&format!("{}-{}", self.id, "min_value")),
            left_label: Some(String::from("↓")),
            min: 0.0,
            max: initial_config.max_value,
            of_universe: (move |universe| {
                universe
                    .circle_config
                    .color_config
                    .saturation_config
                    .min_value
            }),
            on_update: (move |universe, value| {
                universe
                    .circle_config
                    .color_config
                    .saturation_config
                    .min_value = value
            }),
            step: self.step,
            title: String::from("Saturation min_value"),
        };

        let min_value_slider = SliderConfig::create_slider(&min_value_slider_config, &universe);

        let max_value_slider_config = SliderConfig {
            id: String::from(&format!("{}-{}", self.id, "max_value")),
            left_label: Some(String::from("↑")),
            min: initial_config.min_value,
            max: initial_config.max_value,
            of_universe: (move |universe| {
                universe
                    .circle_config
                    .color_config
                    .saturation_config
                    .max_value
            }),
            on_update: (move |universe, value| {
                universe
                    .circle_config
                    .color_config
                    .saturation_config
                    .max_value = value
            }),
            step: self.step,
            title: String::from("Saturation max_value"),
        };

        let max_value_slider = SliderConfig::create_slider(&max_value_slider_config, &universe);

        div.set_inner_text("Saturation");
        div.append_child(&delta_slider).unwrap();
        div.append_child(&min_value_slider).unwrap();
        div.append_child(&max_value_slider).unwrap();
        div
    }

    fn lightness_create_section_slider(
        &self,
        universe: &Arc<Mutex<Universe>>,
    ) -> web_sys::HtmlDivElement {
        let div = new_control_div();
        let initial_config = (self.of_universe)(&universe.lock().unwrap());
        let delta_slider_config = SliderConfig {
            id: String::from(&format!("{}-{}", self.id, "delta")),
            left_label: Some(String::from("Δ")),
            min: 0.0,
            max: initial_config.max_value - initial_config.min_value,
            of_universe: (move |universe| {
                universe
                    .circle_config
                    .color_config
                    .lightness_config
                    .max_delta
            }),
            on_update: (move |universe, value| {
                universe
                    .circle_config
                    .color_config
                    .lightness_config
                    .max_delta = value
            }),
            step: self.step,
            title: String::from("Lightness delta"),
        };

        let delta_slider = SliderConfig::create_slider(&delta_slider_config, &universe);

        let min_value_slider_config = SliderConfig {
            id: String::from(&format!("{}-{}", self.id, "min_value")),
            left_label: Some(String::from("↓")),
            min: 0.0,
            max: 1.0,
            of_universe: (move |universe| {
                universe
                    .circle_config
                    .color_config
                    .lightness_config
                    .min_value
            }),
            on_update: (move |universe, value| {
                universe
                    .circle_config
                    .color_config
                    .lightness_config
                    .min_value = value
            }),
            step: self.step,
            title: String::from("Lightness min_value"),
        };

        let min_value_slider = SliderConfig::create_slider(&min_value_slider_config, &universe);

        let max_value_slider_config = SliderConfig {
            id: String::from(&format!("{}-{}", self.id, "max_value")),
            left_label: Some(String::from("↑")),
            min: initial_config.min_value,
            max: initial_config.max_value,
            of_universe: (move |universe| {
                universe
                    .circle_config
                    .color_config
                    .lightness_config
                    .max_value
            }),
            on_update: (move |universe, value| {
                universe
                    .circle_config
                    .color_config
                    .lightness_config
                    .max_value = value
            }),
            step: self.step,
            title: String::from("Lightness max_value"),
        };

        let max_value_slider = SliderConfig::create_slider(&max_value_slider_config, &universe);

        div.set_inner_text("Lightness");
        div.append_child(&delta_slider).unwrap();
        div.append_child(&min_value_slider).unwrap();
        div.append_child(&max_value_slider).unwrap();
        div
    }

    fn create_hue_section(universe: &Arc<Mutex<Universe>>) -> web_sys::HtmlDivElement {
        let config = ColorParamConfigSliderConfig {
            id: String::from("hue-config"),
            of_universe: (|universe| universe.circle_config.color_config.hue_config),
            step: 1.,
        };
        config.hue_create_section_slider(universe)
    }

    fn create_saturation_section(universe: &Arc<Mutex<Universe>>) -> web_sys::HtmlDivElement {
        let config = ColorParamConfigSliderConfig {
            id: String::from("saturation-config"),
            of_universe: (|universe| universe.circle_config.color_config.saturation_config),
            step: 0.05,
        };
        config.saturation_create_section_slider(universe)
    }

    fn create_lightness_section(universe: &Arc<Mutex<Universe>>) -> web_sys::HtmlDivElement {
        let config = ColorParamConfigSliderConfig {
            id: String::from("lightness-config"),
            of_universe: (|universe| universe.circle_config.color_config.lightness_config),

            step: 0.05,
        };
        config.lightness_create_section_slider(universe)
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

fn new_control_div() -> web_sys::HtmlDivElement {
    let div = document()
        .create_element("div")
        .unwrap()
        .dyn_into::<web_sys::HtmlDivElement>()
        .unwrap();

    div.set_class_name("control");
    div
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

enum ButtonText {
    STATIC(String),
    DYNAMIC(fn(&Universe) -> String),
}

struct ButtonConfig {
    id: String,
    text: ButtonText,
    on_click: fn(&mut Universe, &mut web_sys::SvgElement) -> (),
}

impl ButtonConfig {
    pub fn new_button(
        self,
        universe: &Arc<Mutex<Universe>>,
        svg: &Arc<Mutex<web_sys::SvgElement>>,
    ) -> web_sys::HtmlButtonElement {
        let button = document()
            .create_element("button")
            .unwrap()
            .dyn_into::<web_sys::HtmlButtonElement>()
            .unwrap();

        button.set_id(&self.id);
        let inner_text: String = match &self.text {
            ButtonText::STATIC(text) => String::from(text),
            ButtonText::DYNAMIC(f) => (f)(&universe.lock().unwrap()),
        };
        button.set_inner_text(&inner_text);

        let universe_clone = Arc::clone(&universe);
        let svg_clone = Arc::clone(&svg);
        let on_click_handler = Closure::wrap(Box::new(move || {
            web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
                "You pushed a button!",
            )));

            (self.on_click)(
                &mut universe_clone.lock().unwrap(),
                &mut svg_clone.lock().unwrap(),
            );

            match self.text {
                ButtonText::STATIC(_) => {}
                ButtonText::DYNAMIC(f) => {
                    let button = document()
                        .get_element_by_id(&self.id)
                        .unwrap()
                        .dyn_into::<web_sys::HtmlButtonElement>()
                        .unwrap();

                    button.set_inner_text(&(f)(&universe_clone.lock().unwrap()));
                }
            };

            indicate_next_step(universe_clone.lock().unwrap().is_empty());
        }) as Box<dyn FnMut()>);

        button.set_onclick(Some(on_click_handler.as_ref().unchecked_ref()));
        on_click_handler.forget();

        button
    }
}

struct CheckboxConfig {
    id: String,
    text: String,
    on_click: fn(&mut Universe, bool) -> (),
}

impl CheckboxConfig {
    fn new_checkbox(self, universe: &Arc<Mutex<Universe>>) -> web_sys::HtmlDivElement {
        let checkbox = document()
            .create_element("input")
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();

        checkbox.set_id(&self.id);
        checkbox.set_type("checkbox");

        let div = control_div(&checkbox, &self.id, Some(&self.text));
        let universe_clone = Arc::clone(&universe);
        let on_click_handler = Closure::wrap(Box::new(move || {
            let is_checked = document()
                .get_element_by_id(&self.id)
                .unwrap()
                .dyn_into::<web_sys::HtmlInputElement>()
                .unwrap()
                .checked();
            (self.on_click)(&mut universe_clone.lock().unwrap(), is_checked);
        }) as Box<dyn FnMut()>);

        checkbox.set_onclick(Some(on_click_handler.as_ref().unchecked_ref()));
        on_click_handler.forget();
        div
    }
}

// fn add_on_click_handler(universe: &Arc<Mutex<Universe>>) {
//     let id = "special-slider";
//     let slider = document().get_element_by_id(id).unwrap();

//     let universe_clone = Arc::clone(&universe);
//     let on_change_handler = Closure::wrap(Box::new(move || {
//         web_sys::console::log(&js_sys::Array::from(&JsValue::from_str(
//             "You did a thing!!",
//         )));
//         // let value = document()
//         //     .get_element_by_id(&id)
//         //     .unwrap()
//         //     .dyn_into::<web_sys::HtmlElement>()
//         //     .unwrap()
//         //     .checked();
//         // (self.on_change)(&mut universe_clone.lock().unwrap(), is_checked);
//     }) as Box<dyn FnMut()>);

//     slider.set_onchange(Some(on_change_handler.as_ref().unchecked_ref()));
//     on_change_handler.forget();
// }

fn indicate_next_step(no_circles: bool) {
    let class_name = if no_circles { "highlight" } else { "" };
    for button_id in vec![ADD_BUTTON_ID, APPLE_BUTTON_ID] {
        document()
            .get_element_by_id(&button_id)
            .unwrap()
            .set_class_name(class_name);
    }
}

fn update_canvas_size(height: f64, width: f64) {
    for canvas in all_canvases() {
        canvas.set_height(height as u32);
        canvas.set_width(width as u32);
    }
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let width = body().client_width();
    let height = body().client_height();
    update_canvas_size(height.into(), width.into());
    let universe = Arc::new(Mutex::new(Universe {
        config: Config {
            status: Status::RUNNING,
            speed: Speed::NORMAL,
            bug_checkbox: false,
            radius: 10.,
            apple_steps: 1000,
            initial_height: height as f64,
            initial_width: width as f64,
            color_mode: ColorMode::HSL,
        },
        circle_config: CircleConfig {
            height: height as f64,
            width: width as f64,
            max_position_delta: 6.3,
            color_config: ColorConfig {
                hue_config: ColorParamConfig {
                    max_delta: 2.,
                    min_value: 0.,
                    max_value: 360.,
                },

                saturation_config: ColorParamConfig {
                    max_delta: 0.05,
                    min_value: 0.0,
                    max_value: 1.,
                },

                lightness_config: ColorParamConfig {
                    max_delta: 0.05,
                    min_value: 0.0,
                    max_value: 1.0,
                },
            },
        },
        circles: vec![],
        apples: vec![],
    }));

    let svg = document()
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")
        .unwrap()
        .dyn_into::<web_sys::SvgElement>()
        .map_err(|_| ())
        .unwrap();

    svg.set_id("svg");
    svg.set_attribute("width", &width.to_string())?;
    svg.set_attribute("height", &height.to_string())?;
    svg.set_attribute("viewBox", &format!("0 0 {} {}", width, height))?;

    let svg = Arc::new(Mutex::new(svg));

    let distance_slider_id = "distance-slider";
    let distance_slider_config = SliderConfig {
        id: String::from(distance_slider_id),
        title: String::from("Movement Speed"),

        left_label: Some(String::from("↔")),
        min: 0.0,

        max: 100.0,
        step: 1.0,
        of_universe: (move |universe| universe.circle_config.max_position_delta),
        on_update: (move |universe, value| universe.circle_config.max_position_delta = value),
    };

    let distance_slider_div = SliderConfig::create_slider(&distance_slider_config, &universe);

    let radius_slider_id = "radius-slider";
    let radius_slider_config = SliderConfig {
        id: String::from(radius_slider_id),
        title: String::from("Size"),
        left_label: None,
        min: 1.0,
        max: 100.0,
        step: 1.0,
        of_universe: (move |universe| universe.config.radius),
        on_update: (move |universe, value| universe.config.radius = value),
    };

    let add_button_config = ButtonConfig {
        id: String::from(ADD_BUTTON_ID),
        text: ButtonText::STATIC(String::from("+")),
        on_click: (move |universe, _svg| {
            universe.add_circle();

            document()
                .get_element_by_id(ADD_BUTTON_ID)
                .unwrap()
                .set_class_name("");
        }),
    };
    let add_button = add_button_config.new_button(&universe, &svg);

    let new_circle_div = SliderConfig::create_slider(&radius_slider_config, &universe);

    new_circle_div.append_child(&add_button)?;

    let freeze_button_config = ButtonConfig {
        id: String::from("freeze-button"),
        text: ButtonText::STATIC(String::from("🧊")),

        on_click: (move |universe, _svg| {
            universe.circles.clear();
        }),
    };

    let freeze_button = ButtonConfig::new_button(freeze_button_config, &universe, &svg);

    let apple_steps_slider_config = SliderConfig {
        id: String::from("apple-steps-slider"),
        title: String::from("Steps"),
        left_label: Some(String::from("👣")),
        min: 0.0,
        max: 10000.0,
        step: 100.0,
        of_universe: (move |universe| universe.config.apple_steps as f64),
        on_update: (move |universe, value| universe.config.apple_steps = value as u32),
    };

    let apple_button_config = ButtonConfig {
        id: String::from("apple-button"),
        text: ButtonText::STATIC(String::from("🍏")),

        on_click: (move |universe, _svg| {
            universe.add_apple();
        }),
    };

    let new_apple_div = SliderConfig::create_slider(&apple_steps_slider_config, &universe);

    let apple_button = ButtonConfig::new_button(apple_button_config, &universe, &svg);
    new_apple_div.append_child(&apple_button)?;

    let start_stop_button_id = "start-stop-button";
    let start_stop_button_config = ButtonConfig {
        id: String::from(start_stop_button_id),
        text: ButtonText::DYNAMIC(move |universe| universe.config.status.to_button_display()),
        on_click: move |universe, _svg| {
            universe.config.status.toggle();
        },
    };
    let start_stop_button = start_stop_button_config.new_button(&universe, &svg);

    let speed_button_id = "speed-button";
    let speed_button_config = ButtonConfig {
        id: String::from(speed_button_id),
        text: ButtonText::DYNAMIC(move |universe| universe.config.speed.to_button_display()),
        on_click: (move |universe, _svg| {
            universe.config.speed.toggle();
        }),
    };
    let speed_button = speed_button_config.new_button(&universe, &svg);

    let trash_button_config = ButtonConfig {
        id: String::from("trash-button"),
        text: ButtonText::STATIC(String::from("🗑️")),
        on_click: (move |universe, svg| {
            universe.circles.clear();
            universe.apples.clear();
            clear_board(svg);
            document()
                .get_element_by_id(ADD_BUTTON_ID)
                .unwrap()
                .set_class_name("highlight");
        }),
    };
    let trash_button = trash_button_config.new_button(&universe, &svg);

    let save_button_config = ButtonConfig {
        id: String::from("save-button"),
        text: ButtonText::STATIC(String::from("💾")),
        on_click: (move |_universe, svg| {
            let xml_serializer = web_sys::XmlSerializer::new().unwrap();
            let svg_buf = xml_serializer.serialize_to_string(&svg).unwrap();
            let mut blob_type = web_sys::BlobPropertyBag::new();
            blob_type.type_("image/svg+xml;charset=utf-8");

            let arr = js_sys::Array::new_with_length(1);
            arr.set(0, JsValue::from_str(&svg_buf));

            let blob =
                web_sys::Blob::new_with_str_sequence_and_options(&JsValue::from(arr), &blob_type)
                    .unwrap();

            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
            let anchor = document()
                .create_element("a")
                .unwrap()
                .dyn_into::<web_sys::HtmlAnchorElement>()
                .unwrap();

            anchor.set_href(&url);
            anchor.set_download("art.svg");
            anchor.click();
        }),
    };
    let save_button = save_button_config.new_button(&universe, &svg);

    let bug_checkbox_config = CheckboxConfig {
        id: String::from("bug-checkbox"),
        text: String::from("🐛"),
        on_click: (move |universe, value| {
            universe.config.bug_checkbox = value;
        }),
    };
    let bug_checkbox = bug_checkbox_config.new_checkbox(&universe);

    body().append_child(&start_stop_button)?;
    body().append_child(&speed_button)?;
    body().append_child(&freeze_button)?;
    body().append_child(&save_button)?;
    body().append_child(&trash_button)?;
    body().append_child(&new_circle_div)?;
    body().append_child(&new_apple_div)?;
    body().append_child(&bug_checkbox)?;
    body().append_child(&distance_slider_div)?;
    // body().append_child(&color_slider_div)?;

    body().append_child(&ColorParamConfigSliderConfig::create_hue_section(&universe))?;

    body().append_child(&ColorParamConfigSliderConfig::create_saturation_section(
        &universe,
    ))?;
    body().append_child(&ColorParamConfigSliderConfig::create_lightness_section(
        &universe,
    ))?;
    body().append_child(&svg.lock().unwrap())?;

    universe.lock().unwrap().add_circle();

    let main_loop = Rc::new(RefCell::new(None));
    let main_loop_copy = main_loop.clone();

    clear_board(&mut svg.lock().unwrap());

    let svg_clone = svg.clone();
    *main_loop_copy.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let steps = universe.lock().unwrap().steps();

        let mut universe = universe.lock().unwrap();
        for _ in 0..steps {
            universe.tick();

            // Long term don't want render to mutate universe. Something to think about
            render_svg(&mut universe, &svg_clone.lock().unwrap());
        }

        request_animation_frame(main_loop.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(main_loop_copy.borrow().as_ref().unwrap());
    Ok(())
}
