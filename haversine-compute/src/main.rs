use clap::Parser;
use haversine_compute::{compute_haversine, Point};
use instrument::cpu_timer::{estimate_cpu_frequency, read_cpu_timer};
use json_parser::parser::JsonParser;
use json_parser::value::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct HaversineCompute {
    input: String,
    answers: Option<String>,
}

#[derive(Default, Copy, Clone)]
pub struct Metrics {
    startup: u64,
    read: u64,
    setup: u64,
    parse: u64,
    sum: u64,
    output: u64,
    total: u64,
}

impl Metrics {
    fn print_metric(label: &str, value: u64, total: u64) {
        let percentage = 100.0 * value as f64 / total as f64;

        println!("{label}: {value} ({percentage:.4}%)");
    }
}

fn main() {
    let start = read_cpu_timer();

    let mut metrics = Metrics::default();
    let HaversineCompute { input, answers } = HaversineCompute::parse();

    metrics.startup = read_cpu_timer();

    let file = File::open(input).unwrap();
    let mut answers_file = answers.map(|answers| File::open(answers).unwrap());

    metrics.read = read_cpu_timer();

    let mut answers = String::new();
    let answers = if let Some(answers_file) = &mut answers_file {
        answers_file.read_to_string(&mut answers).unwrap();

        answers
            .split('\n')
            .filter_map(|line| line.parse::<f64>().ok())
            .collect()
    } else {
        vec![]
    };

    metrics.setup = read_cpu_timer();

    let json_value = JsonParser::parse(file).unwrap();

    let points: &HashMap<String, Value> = (&json_value).try_into().unwrap();
    let pairs: &Vec<Value> = points.get("pairs").unwrap().try_into().unwrap();

    metrics.parse = read_cpu_timer();

    let mut sum = 0.;

    for (index, point) in pairs.iter().enumerate() {
        if let Value::Object(object) = point {
            let x0: f64 = object.get("x0").unwrap().try_into().unwrap();
            let x1: f64 = object.get("x1").unwrap().try_into().unwrap();
            let y0: f64 = object.get("y0").unwrap().try_into().unwrap();
            let y1: f64 = object.get("y1").unwrap().try_into().unwrap();

            let result = compute_haversine(Point { x0, y0, x1, y1 }, 6372.8);
            sum += result;

            if let Some(answer) = answers.get(index) {
                assert_eq!(*answer, result, "Difference: {}", *answer - result);
            }
        }
    }

    metrics.sum = read_cpu_timer();

    println!("Average distance: {}", sum / pairs.len() as f64);

    metrics.output = read_cpu_timer();
    metrics.total = read_cpu_timer();

    let cpu_frequency = estimate_cpu_frequency();

    println!(
        "Total time: {:.4}ms",
        1000. * (metrics.total - start) as f64 / cpu_frequency as f64
    );

    Metrics::print_metric("Startup", metrics.startup - start, metrics.total - start);
    Metrics::print_metric(
        "Read",
        metrics.read - metrics.startup,
        metrics.total - start,
    );
    Metrics::print_metric("Setup", metrics.setup - metrics.read, metrics.total - start);
    Metrics::print_metric(
        "Parse",
        metrics.parse - metrics.setup,
        metrics.total - start,
    );
    Metrics::print_metric("Sum", metrics.sum - metrics.parse, metrics.total - start);
    Metrics::print_metric(
        "Output",
        metrics.output - metrics.sum,
        metrics.total - start,
    );
}
