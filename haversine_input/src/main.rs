use crate::clustered::ClusteredHaversinePointsGenerator;
use crate::types::HaversinePointGenerator;
use crate::uniform::UniformHaversinePointsGenerator;
use clap::{Parser, Subcommand};
use json_parser::tokens::JsonParser;
use std::fs::File;

mod clustered;
mod formula;
mod types;
mod uniform;

#[derive(Subcommand, Debug)]
pub enum Method {
    Uniform { seed: String, points_count: usize },
    Cluster { seed: String, points_count: usize },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct HaversineInput {
    #[command(subcommand)]
    method: Method,
}

fn main() {
    let input = HaversineInput::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &input.method {
        Method::Uniform { seed, points_count } => {
            let generator = UniformHaversinePointsGenerator {};
            let mut output = File::create("./test.json").unwrap();

            let average_distance = generator
                .generate(seed.clone(), *points_count, &mut output)
                .unwrap();

            drop(output);

            println!("Average distances: {average_distance}");

            let mut json_parser = JsonParser::new(File::open("./test.json").unwrap());
            let value = json_parser.parse_json().unwrap();

            println!("{:#?}", value);
        }
        Method::Cluster { seed, points_count } => {
            let generator = ClusteredHaversinePointsGenerator {};
            let mut output = File::create("./test.json").unwrap();

            let average_distance = generator
                .generate(seed.clone(), *points_count, &mut output)
                .unwrap();

            drop(output);

            println!("Average distances: {average_distance}");

            let mut json_parser = JsonParser::new(File::open("./test.json").unwrap());
            let value = json_parser.parse_json().unwrap();

            println!("{:#?}", value);
        }
    }
}
