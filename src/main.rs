use std::{error::Error, time::Instant, io::Write};

use crate::{io::read_all, sim::SimulationState};

pub(crate) mod io;
pub(crate) mod model;
pub(crate) mod render;
pub(crate) mod sim;

fn main() -> Result<(), Box<dyn Error>> {
    let node_file = std::fs::File::open("locations.csv")?;
    let relations_file = std::fs::File::open("rail.csv")?;
    let (nodes, relations) = read_all(node_file, relations_file)?;

    let state = SimulationState::new(nodes, relations);

    let start = Instant::now();
    let last_change = state.run_n_steps(20000)?;
    let elapsed = start.elapsed();
    println!("Elapsed => {:?} Last Change => {last_change}", elapsed);

    let rendered = state.render(1000.0, 1000.0);

    let mut out_file = std::fs::File::create("out.svg")?;
    out_file.write_all(rendered.as_bytes())?;

    Ok(())
}
