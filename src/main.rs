use std::{error::Error, time::Instant};

use crate::{io::read_all, model::Coordinates, sim::SimulationState};

pub(crate) mod io;
pub(crate) mod model;
pub(crate) mod sim;

fn main() -> Result<(), Box<dyn Error>> {
    let node_file = std::fs::File::open("locations.csv")?;
    let relations_file = std::fs::File::open("rail.csv")?;
    let (nodes, relations) = read_all(node_file, relations_file)?;

    let state = SimulationState::new(nodes, relations);

    let start = Instant::now();
    let last_change = state.run_n_steps(100)?;
    let elapsed = start.elapsed();
    println!("Elapsed => {:?} Last Change => {last_change}", elapsed);

    Ok(())
}
