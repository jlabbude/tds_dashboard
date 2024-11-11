use yew::prelude::*;
use web_sys::HtmlCanvasElement;
use wasm_bindgen::{JsCast, JsValue};
use yew_hooks::use_interval;
use std::collections::VecDeque;
use rand::Rng;

#[derive(Clone, Debug, PartialEq, Copy)]
struct TdsDataPoint {
    value: f64,
    timestamp: f64,
}

#[derive(Properties, PartialEq, Clone)]
struct TdsGraphProps {
    value: f64,
    history: VecDeque<TdsDataPoint>
}

#[hook]
fn use_fetch_data() -> UseStateHandle<f64> {
    let data: f64 = rand::thread_rng().gen_range(0.0..1000.0);
    log::info!("DATA {data}");
    use_state(|| data)
}

fn use_tds_history(history: &VecDeque<TdsDataPoint>, current_value: f64) -> VecDeque<TdsDataPoint> {
    log::warn!("before {history:?}");
    let mut new_history = history.clone();
    if new_history.len() > 60 {
        new_history.pop_front();
    }
    new_history.push_back(TdsDataPoint {
        value: current_value,
        timestamp: web_sys::js_sys::Date::now(),
    });
    log::warn!("after {history:?}");
    new_history.clone()
}

#[function_component]
fn TdsGraph(props: &TdsGraphProps) -> Html {
    let canvas_ref = use_node_ref();
    let history = &props.history;
    log::info!("{history:?}");

    {
        let canvas_ref = canvas_ref.clone();
        let history = history.clone();
        use_effect_with(history.clone(), move |_| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let context = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<web_sys::CanvasRenderingContext2d>()
                    .unwrap();

                let canvas_width = canvas.width() as f64;
                let canvas_height = canvas.height() as f64;

                context.clear_rect(0.0, 0.0, canvas_width, canvas_height);

                if !history.is_empty() {
                    let max_value = history.iter().map(|p| p.value).fold(f64::NEG_INFINITY, f64::max);
                    let min_value = history.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);

                    context.begin_path();
                    context.set_stroke_style(&JsValue::from_str("#2196F3"));
                    context.set_line_width(2.0);

                    for (i, point) in history.iter().enumerate() {
                        let x = (i as f64 / history.len() as f64) * canvas_width;
                        let y = canvas_height - ((point.value - min_value) / (max_value - min_value)) * canvas_height;

                        if i == 0 {
                            context.move_to(x, y);
                        } else {
                            context.line_to(x, y);
                        }
                    }

                    context.stroke();
                }
            }
            || ()
        });
    }

    html! {
        <canvas
            ref={canvas_ref}
            width="600"
            height="200"
            style="border: 1px solid #ccc; background-color: #f5f5f5;"
        />
    }
}

#[function_component]
fn App() -> Html {
    let data = use_state(|| use_fetch_data());
    let history = use_state(|| VecDeque::with_capacity(60));
    {
        let history = history.clone();
        let data = data.clone();
        use_interval(move || {
            history.set(use_tds_history(&history, *data));
            data.set(use_fetch_data());
        }, 1000);
    }

    let get_quality_level = |tds: f64| -> (&'static str, &'static str) {
        match tds {
            x if x < 50.0 => ("Excellent", "#4CAF50"),
            x if x < 150.0 => ("Good", "#8BC34A"),
            x if x < 250.0 => ("Fair", "#FFC107"),
            x if x < 350.0 => ("Poor", "#FF9800"),
            _ => ("Unsafe", "#F44336"),
        }
    };

    let (quality, color) = get_quality_level(*data);

    html! {
    <div class="container">
        <h1>{"Water Quality Dashboard"}</h1>
        
        <div class="grid-container">
            <div class="card">
                <h2>{"Current TDS"}</h2>
                <div class="value">
                    {format!("{:.1} ppm", *data)}
                </div>
            </div>
            
            <div class="card">
                <h2>{"Water Quality"}</h2>
                <div class="value" style={format!("color: {};", color)}>
                    {quality}
                </div>
            </div>
        </div>
        
        <div class="canvas-container">
            <h2>{"TDS History"}</h2>
            <TdsGraph value={*data} history={(*history).clone()} />
        </div>
    </div>
}
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
