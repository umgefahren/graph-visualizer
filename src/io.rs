use std::{io::Read, sync::Arc};

use nohash_hasher::IntMap;
use rand::{Rng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::model::{Node, Relation};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct NodeRecord {
    id: usize,
    weight: f64,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct RelationRecord {
    id: usize,
    from: usize,
    to: usize,
    weight: f64,
}

fn generate_coordinate<R: RngCore>(mut rng: R) -> f64 {
    loop {
        let tmp: f64 = rng.gen_range(0.0..100.0);
        if tmp.is_normal() {
            return tmp;
        }
    }
}

fn read_nodes<R: Read>(reader: R) -> std::io::Result<IntMap<usize, Arc<Node>>> {
    let mut rdr = csv::Reader::from_reader(reader);

    let mut res = IntMap::default();
    let mut rng = rand::rngs::SmallRng::from_seed([0u8; 32]);

    for result in rdr.deserialize() {
        let record: NodeRecord = result?;
        let id = record.id;
        let (x, y) = (generate_coordinate(&mut rng), generate_coordinate(&mut rng));
        let node = Arc::new(Node::new(record.id, x, y, record.weight));
        res.insert(id, node);
    }

    Ok(res)
}

fn read_relations<R: Read>(
    reader: R,
    nodes: &mut IntMap<usize, Arc<Node>>,
) -> std::io::Result<IntMap<usize, Arc<Relation>>> {
    let mut rdr = csv::Reader::from_reader(reader);

    let mut res = IntMap::default();

    for result in rdr.deserialize() {
        let record: RelationRecord = result?;
        let to_node = nodes[&record.to].clone();
        let from_node = nodes[&record.from].clone();
        let relation = Arc::new(Relation::new(record.weight, from_node, to_node));
        relation.register();
        res.insert(record.id, relation);
    }
    Ok(res)
}

pub fn read_all<N: Read, R: Read>(
    node_reader: N,
    relation_reader: R,
) -> std::io::Result<(Vec<Arc<Node>>, Vec<Arc<Relation>>)> {
    let mut nodes = read_nodes(node_reader)?;
    let relations = read_relations(relation_reader, &mut nodes)?;
    let ret_nodes = nodes.into_values().collect();
    let ret_relations = relations.into_values().collect();
    Ok((ret_nodes, ret_relations))
}
