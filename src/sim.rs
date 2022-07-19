use std::{
    ops::Range,
    sync::{Arc, Barrier, RwLock},
    thread::{self, available_parallelism},
};

use crate::{
    model::{Coordinates, Node, Relation},
    render::{Element, Renderer},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref AVAILABLE_PARALLELISM: usize = available_parallelism().unwrap().get();
}

const SPING_SCALE: f64 = 1.0 / 200.0;
const COLOUMB_SCALE: f64 = 1.0 / 1.0;
const TIME_DELTA: f64 = 0.1;

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

    fn run_simulation_step(&self, n: usize) -> std::io::Result<f64> {
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
                let mut new_coordinates: Vec<Coordinates> = Vec::new();
                for _ in 0..n {
                    let iterator = local_nodes[range.clone()]
                        .iter()
                        .map(|e| (e, e.loc.read().unwrap().clone()))
                        .map(|(e, c)| {
                            (
                                e.calc_new_position(&local_nodes, SPING_SCALE, COLOUMB_SCALE, TIME_DELTA),
                                c,
                            )
                        })
                        .map(|(new, old)| (new, new.to(old).length()))
                        .map(|(new, length)| {
                            if length.is_normal() {
                                total_length += length.abs();
                            }
                            new
                        });
                    new_coordinates.clear();
                    new_coordinates.extend(iterator);
                    local_barrier.wait();
                    local_nodes[range.clone()]
                        .iter()
                        .zip(new_coordinates.iter())
                        .for_each(|(n, c)| n.update_coordinates(*c));
                    local_barrier.wait();
                }
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
        self.run_simulation_step(n)
    }

    pub fn render(self, x: f64, y: f64) -> String {
        let mut renderer = Renderer::new();
        self.nodes
            .iter()
            .map(|e| Element::from(e.as_ref()))
            .for_each(|e| renderer.add_element(e));

        self.relations
            .iter()
            .map(|e| Element::from(e.as_ref()))
            .for_each(|e| renderer.add_element(e));

        renderer.render(x, y)
    }
}
