#![allow(deprecated, clippy::redundant_closure)]
use chrono::{DateTime, Utc};
use rand::Rng;
use serde_json::json;
use std::collections::VecDeque;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_hooks::use_interval;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/package.js")]
extern "C" {
    pub fn createChart(canvas_id: &str, data: JsValue, options: JsValue) -> JsValue;
    pub fn updateChart(canvas_id: &str, new_data: JsValue);
    #[wasm_bindgen(js_name = chartInstances)]
    static CHART_INSTANCES: JsValue;
}

#[derive(Clone, Debug, PartialEq, Copy)]
struct TdsDataPoint {
    value: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Properties, PartialEq, Clone)]
struct TdsGraphProps {
    history: VecDeque<TdsDataPoint>,
}

pub fn is_chart_instances_empty() -> bool {
    js_sys::Object::keys(CHART_INSTANCES.dyn_ref::<js_sys::Object>()
        .expect("CHART_INSTANCES is not an object"))
        .length() == 0
}

fn fetch_data() -> f64 {
    let data: f64 = rand::thread_rng().gen_range(0.0..1000.0);
    log::info!("DATA {data}");
    data
}

fn tds_history(old_history: &VecDeque<TdsDataPoint>, current_value: f64) -> VecDeque<TdsDataPoint> {
    let mut new_history = old_history.clone();
    if new_history.len() > 60 {
        new_history.pop_front();
    }
    new_history.push_back(TdsDataPoint {
        value: current_value,
        timestamp: Utc::now(),
    });
    new_history.clone()
}

#[function_component]
fn Graph(props: &TdsGraphProps) -> Html {
    use wasm_bindgen::JsValue;
    use web_sys::HtmlCanvasElement;
    

    let canvas_ref = use_node_ref(); // Reference to the canvas element

    {
        let canvas_ref = canvas_ref.clone();
        let history = props.history.clone();

        use_effect_with(
            props.history.clone(), // Re-run effect when history updates
            move |_| {
                // Prepare the data for the chart
                let labels: Vec<String> = history
                    .iter()
                    .map(|point| point.timestamp.format("%H:%M:%S").to_string())
                    .collect();
                let data: Vec<f64> = history.iter().map(|point| point.value).collect();

                let chart_data = json!({
                    "labels": labels,
                    "datasets": [{
                        "label": "TDS (ppm)",
                        "data": data,
                        "fill": false,
                        "borderColor": "rgba(75, 192, 192, 1)",
                        "tension": 0.1
                    }]
                });

                let options = json!({
                    "scales": {
                        "y": {
                            "beginAtZero": true
                        }
                    }
                });

                // Get the canvas element
                if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                    match is_chart_instances_empty() {
                        true => { createChart(&canvas.id(), JsValue::from_serde(&chart_data).unwrap(), JsValue::from_serde(&options).unwrap()); },
                        false => updateChart(&canvas.id(), JsValue::from_serde(&chart_data).unwrap()),
                    }
                }

                || ()
            },
        );
    }

    html! {
        <canvas id="tdsGraph" ref={canvas_ref}></canvas>
    }
}


#[function_component]
fn App() -> Html {
    let data = use_state(|| fetch_data());
    let history = use_state(|| VecDeque::with_capacity(60));
    {
        let history = history.clone();
        let data = data.clone();
        use_interval(
            move || {
                history.set(tds_history(&history, *data));
                data.set(fetch_data());
            },
            1000,
        );
    }

    let get_quality_level = |tds: f64| -> (&'static str, &'static str) {
        match tds {
            0.0..100.0 => ("Excelente", "#4CAF50"),
            100.0..200.0 => ("Bom", "#8BC34A"),
            200.0..300.0 => ("Aceitável", "#FFC107"),
            300.0..400.0 => ("Ruim", "#FF9800"),
            400.0..500.0 => ("Péssimo", "#FF9800"),
            _ => ("Perigoso", "#F44336"),
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
                <Graph history={(*history).clone()}/>
                </div>
                </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
