use std::{
    ops::Range,
    sync::{Arc, Barrier},
    thread::{self, available_parallelism},
};

use crate::model::{Coordinates, Node, Relation};

use lazy_static::lazy_static;

lazy_static! {
    static ref AVAILABLE_PARALLELISM: usize = available_parallelism().unwrap().get();
}

pub struct SimulationState {
    nodes: Arc<Vec<Arc<Node>>>,
    relations: Arc<Vec<Arc<Relation>>>,
}

impl SimulationState {
    pub fn new(nodes: Vec<Arc<Node>>, relations: Vec<Arc<Relation>>) -> Self {
        Self {
            nodes: Arc::new(nodes),
            relations: Arc::new(relations),
        }
    }

    fn run_simulation_step(&self) -> std::io::Result<f64> {
        let thread_nums = *AVAILABLE_PARALLELISM;
        let nodes_len = self.nodes.len();
        let slice_len = nodes_len / thread_nums;
        let ranges: Vec<Range<usize>> = (0..thread_nums)
            .map(|e| {
                if e == 0 {
                    0..slice_len
                } else if e == thread_nums - 1 {
                    (slice_len * e)..nodes_len
                } else {
                    (e * slice_len)..((e + 1) * slice_len)
                }
            })
            .collect();

        let mut handles = Vec::new();
        let barrier = Arc::new(Barrier::new(thread_nums));

        for range in ranges {
            let local_nodes = Arc::clone(&self.nodes);
            let local_barrier = barrier.clone();
            let handle = thread::spawn(move || {
                let mut total_length = 0.0;
                let new_coordinates: Vec<Coordinates> = local_nodes[range.clone()]
                    .iter()
                    .map(|e| (e, e.loc.read().unwrap().clone()))
                    .map(|(e, c) | (e.calc_new_position(&local_nodes, 1.0, 1.0, 1.0), c))
                    .map(|(new, old)| (new, new.to(old).length()))
                    .map(|(new, length)| {
                        if length.is_normal() {
                            total_length += length.abs();
                        }
                        new
                    })
                    .collect();

                local_barrier.wait();
                local_nodes[range]
                    .iter()
                    .zip(new_coordinates.iter())
                    .for_each(|(n, c)| n.update_coordinates(*c));
                
                total_length
            });
            handles.push(handle);
        }

        let mut change = 0.0;

        for handle in handles {
            change += handle.join().unwrap();
        }

        Ok(change)
    }

    pub fn run_n_steps(&self, n: usize) -> std::io::Result<f64> {
        let mut last_change = 0.0;
        for _ in 0..n {
            last_change = self.run_simulation_step()?;
        }
        Ok(last_change)
    }
}
