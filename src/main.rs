#![allow(deprecated, clippy::redundant_closure)]
use chrono::{DateTime, Local, TimeZone};
use serde_json::{json, Value};
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::Element;
use web_sys::HtmlCanvasElement;
use yew::prelude::*;
use yew_hooks::use_interval;

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
    timestamp: DateTime<Local>,
}

#[derive(Properties, PartialEq, Clone)]
struct TdsGraphProps {
    history: VecDeque<TdsDataPoint>,
}

fn serialize_tds_json(val: &Value) -> TdsDataPoint {
    TdsDataPoint {
        value: val["tds_ppm"].as_f64().unwrap(),
        timestamp: Local
            .timestamp_opt(val["timestamp"].as_i64().unwrap(), 0)
            .earliest()
            .unwrap(),
    }
}

pub fn is_chart_instances_empty() -> bool {
    js_sys::Object::keys(
        CHART_INSTANCES
            .dyn_ref::<js_sys::Object>()
            .expect("CHART_INSTANCES is not an object"),
    )
    .length()
        == 0
}

#[hook]
fn use_fetch_latest_data() -> Option<TdsDataPoint> {
    let latest_data = use_state_eq(|| None);

    {
        let latest_data = latest_data.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(response) = reqwest::get("http://localhost:8000/last_message").await {
                if let Ok(json_data) = response.json::<Value>().await {
                    let new_data = serialize_tds_json(&json_data);
                    latest_data.set(Some(new_data));
                }
            } else {
                latest_data.set(None);
            }
        });
    }

    *latest_data
}

#[hook]
fn use_fetch_history() -> UseStateHandle<VecDeque<TdsDataPoint>> {
    let history = use_state(|| VecDeque::<TdsDataPoint>::with_capacity(60));

    {
        let history = history.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(response) = reqwest::get("http://localhost:8000/tds_history").await {
                    if let Ok(json_data) = response.json::<Vec<Value>>().await {
                        history.set(
                            json_data
                                .iter()
                                .map(serialize_tds_json)
                                .collect::<Vec<TdsDataPoint>>()
                                .into(),
                        );
                    }
                }
            });
        });
    }

    history
}

fn tds_history(
    old_history: &VecDeque<TdsDataPoint>,
    current_value: TdsDataPoint,
) -> VecDeque<TdsDataPoint> {
    let mut new_history = old_history.clone();
    if new_history.len() > 60 {
        new_history.pop_front();
    }
    new_history.push_back(current_value);
    new_history.clone()
}

/// `yew_hooks::use_interval` misbehaves if i dont do this
fn refresh(old_history: &VecDeque<TdsDataPoint>) -> VecDeque<TdsDataPoint> {
    old_history.clone()
}

#[function_component]
fn Graph(props: &TdsGraphProps) -> Html {
    let canvas_ref = use_node_ref();

    {
        let canvas_ref = canvas_ref.clone();
        let history = props.history.clone();

        use_effect_with(props.history.clone(), move |_| {
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

            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                if let Some(id) = canvas
                    .dyn_ref::<Element>()
                    .and_then(|el| el.get_attribute("id"))
                {
                    match is_chart_instances_empty() {
                        true => {
                            createChart(
                                &id,
                                JsValue::from_serde(&chart_data).unwrap(),
                                JsValue::from_serde(&options).unwrap(),
                            );
                        }
                        false => {
                            updateChart(&id, JsValue::from_serde(&chart_data).unwrap());
                        }
                    }
                }
            }

            || ()
        });
    }

    html! {
        <canvas id="tdsGraph" ref={canvas_ref}></canvas>
    }
}

#[function_component]
fn App() -> Html {
    let history = use_fetch_history();
    let data_op = use_fetch_latest_data();
    {
        let history = history.clone();
        use_interval(
            move || {
                if let Some(data) = &data_op {
                    let normalized_timestamp = data.timestamp.timestamp();
                    if history.back().map_or(true, |front| {
                        front.timestamp.timestamp() != normalized_timestamp
                    }) {
                        history.set(tds_history(&history, *data));
                    } else {
                        history.set(refresh(&history));
                    }
                }
            },
            1000,
        );
    }

    let data = data_op.as_ref().map_or(0.0, |data| data.value);
    let get_quality_level = |tds: f64| -> (&'static str, &'static str) {
        match tds {
            0.0 => ("Desconectado", "#000000"),
            1.0..300.0 => ("Excelente", "#4CAF50"),
            300.0..600.0 => ("Bom", "#8BC34A"),
            600.0..900.0 => ("Aceitável", "#FFC107"),
            900.0..1200.0 => ("Ruim", "#FF9800"),
            1200.0.. => ("Inaceitável", "#FF9800"),
            _ => ("Inválido", "#000000"), // negative values
        }
    };

    let (quality, color) = get_quality_level(data);

    html! {
        <div class="container">
            <div class="grid-container">
                <div class="card">
                    <h2>{"TDS Atual"}</h2>
                    <div class="value">
                        {format!("{:.1} ppm", data)}
                    </div>
                </div>

                <div class="card">
                    <h2>{"Qualidade"}</h2>
                    <div class="value" style={format!("color: {};", color)}>
                        {quality}
                    </div>
                </div>
            </div>

            <div class="canvas-container">
                <h2>{"Histórico"}</h2>
                <Graph history={(*history).clone()}/>
                </div>
                </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
