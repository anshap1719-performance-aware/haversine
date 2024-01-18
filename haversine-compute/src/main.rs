use assert_float_eq::{
    afe_abs, afe_absolute_error_msg, afe_is_absolute_eq, assert_float_absolute_eq,
};
use clap::Parser;
use haversine_compute::{compute_haversine, Point};
use json_parser::parser::JsonParser;
use json_parser::value::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::mem::size_of;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct HaversineCompute {
    input: String,
    answers: Option<String>,
}

#[cfg_attr(
    feature = "profile",
    instrument(data_expression = "file.metadata().ok().map_or(0, |file| file.len())")
)]
fn read_json_file(mut file: File) -> Vec<u8> {
    let mut container = Vec::with_capacity(
        file.metadata()
            .ok()
            .map_or(0, |file| usize::try_from(file.len()).unwrap()),
    );

    file.read_to_end(&mut container).unwrap();

    container
}

#[cfg_attr(feature = "profile", instrument)]
fn parse_haversine_pairs(file: File) -> Vec<Value> {
    let json_data = read_json_file(file);
    let json_value = JsonParser::parse_from_bytes(&json_data).unwrap();

    instrument_block!("Lookup & Convert", {
        let points: &HashMap<String, Value> = (&json_value).try_into().unwrap();
        let pairs: &Vec<Value> = points.get("pairs").unwrap().try_into().unwrap();

        pairs.clone()
    })
}

#[cfg_attr(feature = "profile", instrument(main))]
fn main() {
    let HaversineCompute { input, answers } = HaversineCompute::parse();

    let file = File::open(input).unwrap();
    let mut answers_file = answers.map(|answers| File::open(answers).unwrap());

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

    let pairs = parse_haversine_pairs(file);

    let mut sum = 0.;

    instrument_block!(
        "sum_pairs",
        {
            for (index, point) in pairs.iter().enumerate() {
                if let Value::Object(object) = point {
                    let x0: f64 = object.get("x0").unwrap().try_into().unwrap();
                    let x1: f64 = object.get("x1").unwrap().try_into().unwrap();
                    let y0: f64 = object.get("y0").unwrap().try_into().unwrap();
                    let y1: f64 = object.get("y1").unwrap().try_into().unwrap();

                    let result = compute_haversine(Point { x0, y0, x1, y1 }, 6372.8);
                    sum += result;

                    if let Some(answer) = answers.get(index) {
                        assert_float_absolute_eq!(*answer, result, f64::EPSILON);
                    }
                }
            }
        },
        (pairs.len() * size_of::<Value>()) as u64
    );

    println!("Average distance: {}", sum / pairs.len() as f64);
}

#[macro_use]
extern crate instrument_macros;
