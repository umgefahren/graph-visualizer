use std::{error::Error, time::Instant, io::Write};

use clap::Parser;

use crate::{io::read_all, sim::SimulationState, cli::Args};

pub(crate) mod io;
pub(crate) mod model;
pub(crate) mod render;
pub(crate) mod sim;
pub(crate) mod cli;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();

    let node_file = std::fs::File::open(args.nodes_file).expect("Failed to open nodes file");
    let relations_file = std::fs::File::open(args.relations_file).expect("Failed to open relations file");
    let (nodes, relations) = read_all(node_file, relations_file)?;

    let state = SimulationState::new(nodes, relations, args.spring, args.coloumb, args.time);

    let start = Instant::now();
    let last_change = state.run_n_steps(args.steps)?;
    let elapsed = start.elapsed();
    println!("Elapsed => {:?} Last Change => {last_change}", elapsed);

    let rendered = state.render(args.width, args.height);

    let mut out_file = std::fs::File::create(args.out)?;
    out_file.write_all(rendered.as_bytes())?;

    Ok(())
}
