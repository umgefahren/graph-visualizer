use std::path::PathBuf;
use crate::sim::{TIME_DELTA, SPING_SCALE, COLOUMB_SCALE};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// Location of the file containing the nodes
    #[clap(short, long, default_value = "locations.csv")]
    pub nodes_file: PathBuf,
    /// Location of the file containing the relations
    #[clap(short, long, default_value = "rail.csv")]
    pub relations_file: PathBuf,
    /// Location of the file containing the generated csv
    #[clap(short, long, default_value = "out.csv")]
    pub out: PathBuf,
    /// Scaling factor of the springs
    #[clap(short, long, default_value_t = SPING_SCALE)]
    pub spring: f32,
    /// Scaling factor of the coloumb force
    #[clap(short, long, default_value_t = COLOUMB_SCALE)]
    pub coloumb: f32,
    /// Time delta in each computation step
    #[clap(short, long, default_value_t = TIME_DELTA)]
    pub time: f32, 
    #[clap(short, long, default_value_t = 20000)]
    pub steps: usize,
    #[clap(short, long, default_value_t = 1000.0)]
    pub width: f32,
    #[clap(short, long, default_value_t = 1000.0)]
    pub height: f32, 
}
