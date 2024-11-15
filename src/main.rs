use std::{
    rc::Rc,
    collections::VecDeque,
};
use chrono::{DateTime, Duration, Utc};
use yew::prelude::*;
use yew_chart::{
    axis::{Axis, Orientation, Scale},
    linear_axis_scale::LinearScale,
    series::{self, Series, Tooltipper, Type, Labeller},
    time_axis_scale::TimeScale,
};
use yew_hooks::use_interval;
use rand::Rng;

static WIDTH: f32 = 533.0;
static HEIGHT: f32 = 300.0;
static MARGIN: f32 = 50.0;
static TICK_LENGTH: f32 = 10.0;

#[derive(Clone, Debug, PartialEq, Copy)]
struct TdsDataPoint {
    value: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Properties, PartialEq, Clone)]
struct TdsGraphProps {
    history: VecDeque<TdsDataPoint>,
}

fn fetch_data() -> f64 {
    let data: f64 = rand::thread_rng().gen_range(0.0..1000.0);
    log::info!("DATA {data}");
    data
}

fn use_tds_history(old_history: &VecDeque<TdsDataPoint>, current_value: f64) -> VecDeque<TdsDataPoint> {
    let mut history = old_history.clone();
    if history.len() > 60 {
        history.pop_front();
    }
    history.push_back(TdsDataPoint {
        value: current_value,
        timestamp: Utc::now(),
    });
    history.clone()
}

#[function_component]
fn Graph(props: &TdsGraphProps) -> Html {
    let end_date = Utc::now();
    let start_date = match props.history.front() {
        Some(time) => time,
        None => return html!{},
    }.timestamp;
    let timespan = start_date..end_date;
    let circle_text_labeller = Rc::from(series::circle_label()) as Rc<dyn Labeller>;
    // Data set for the graph
    let data_set = Rc::new(
        props.history.iter().map(|point| {
            (
                point.timestamp.timestamp_millis(),
                point.value as f32,
                Some(circle_text_labeller.clone()),
            )
        }).collect::<Vec<_>>()
    );

    // Horizontal (time) and vertical (value) scales
    let h_scale = Rc::new(TimeScale::new(timespan, Duration::hours(1))) as Rc<dyn Scale<Scalar = _>>;
    let v_scale = Rc::new(LinearScale::new(0.0..1000.0, 100.0)) as Rc<dyn Scale<Scalar = _>>;

    let tooltip = Rc::from(series::y_tooltip()) as Rc<dyn Tooltipper<_, _>>;

    html! {
        <svg class="chart" viewBox={format!("0 0 {} {}", WIDTH, HEIGHT)} preserveAspectRatio="none">
            <Series<i64, f32>
                series_type={Type::Line}
                name="TDS_meter"
                data={data_set}
                horizontal_scale={h_scale.clone()}
                horizontal_scale_step={Duration::days(2).num_milliseconds()}
                tooltipper={tooltip.clone()}
                vertical_scale={v_scale.clone()}
                x={MARGIN} y={MARGIN} width={WIDTH - (MARGIN * 2.0)} height={HEIGHT - (MARGIN * 2.0)} />

            <Axis<f32>
                name="tempo"
                orientation={Orientation::Left}
                scale={v_scale}
                x1={MARGIN} y1={MARGIN} xy2={HEIGHT - MARGIN}
                tick_len={TICK_LENGTH}/>

            <Axis<i64>
                name="TDS"
                orientation={Orientation::Bottom}
                scale={h_scale}
                x1={MARGIN} y1={HEIGHT - MARGIN} xy2={WIDTH - MARGIN}
                tick_len={TICK_LENGTH}
                title={"Tempo"} />

        </svg>
    }  
}

#[function_component]
fn App() -> Html {
    let data = use_state(|| fetch_data());
    let history = use_state(|| VecDeque::with_capacity(60));
    {
        let history = history.clone();
        let data = data.clone();
        use_interval(move || {
            history.set(use_tds_history(&history, *data));
            data.set(fetch_data());
        }, 1000);
    }

    let get_quality_level = |tds: f64| -> (&'static str, &'static str) {
        match tds {
            0.0..100.0 => ("Excelente", "#4CAF50"),
            100.0.. 200.0 => ("Bom", "#8BC34A"),
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
