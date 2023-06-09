use rust_distributed_sys_challenge::*;

use rand::{rngs::StdRng, Rng, SeedableRng};

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
    sync::mpsc,
    time::Duration,
};
use uuid::{self, Uuid};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")] // IMPORTANT: returns {type:"echo", echo:"..."}
#[serde(rename_all = "snake_case")]
enum Payload {
    //NOTE: find a way to remove OKs from this enum
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: Uuid,
    },
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, HashSet<String>>,
    },
    TopologyOk,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum GeneratedPayload {
    Share { messages: HashSet<usize> },
    ShareOk { messages: HashSet<usize> },
}

struct BroadcastNode {
    node_id: String,
    local_id: usize,
    messages: HashSet<usize>,
    neighbors: HashSet<String>,
    known_by_node: HashMap<String, HashSet<usize>>,
}

/// Generate a small world topology
///
/// args:
///    - `num_nodes`: number of nodes in the network
///    - `local_cluster_count`: number of nodes in each local cluste
///    - `rewire_probability`: probability of rewiring (connecting outside of nearest k)
///
/// returns:
///   - `HashMap<String, HashSet<String>>`: map of node to neighbors
fn generate_small_world_toplogy(
    num_nodes: usize,
    local_cluster_count: usize,
    rewire_probability: f32,
) -> HashMap<String, HashSet<String>> {
    // initial setup
    let mut nodes = HashMap::<String, HashSet<String>>::new();
    let mut rng = StdRng::seed_from_u64(1);
    let num_neighbors = num_nodes / local_cluster_count;
    let beta = rewire_probability; // probability of rewiring (connecting outside of nearest k)

    // NOTE: generate nodes
    for i in 0..num_nodes {
        nodes
            .entry(format!("n{}", i))
            .or_insert(HashSet::<String>::new());
    }

    // NOTE: every node is a neighbor of the nearest `k` nodes
    for i in 0..num_nodes {
        for j in 1..num_neighbors + 1 {
            let neighbor = (i + j) % num_nodes; // easy way to wrap around
            let ith_node = format!("n{}", i);
            let neighbor_node = format!("n{}", neighbor);
            nodes
                .get_mut(&ith_node)
                .unwrap()
                .insert(neighbor_node.clone());
            nodes.get_mut(&neighbor_node).unwrap().insert(ith_node);
        }
    }
    // NOTE: rewire edges from each node
    // slightly different from the original watts-strogatz algorithm but should be equivalent
    for i in 0..num_nodes {
        for j in 0..num_nodes {
            if i < j && rng.gen::<f32>() < beta {
                let ith_node = format!("n{}", i);
                let jth_node = format!("n{}", j);
                nodes.get_mut(&ith_node).unwrap().remove(&jth_node);
                nodes.get_mut(&jth_node).unwrap().remove(&ith_node);

                let new_neighbor = rng.gen_range(0..num_nodes);
                let new_neighbor_node = format!("n{}", new_neighbor);
                nodes
                    .get_mut(&ith_node)
                    .unwrap()
                    .insert(new_neighbor_node.clone());
                nodes.get_mut(&new_neighbor_node).unwrap().insert(ith_node);
            }
        }
    }
    return nodes;
}

// NOTE: state machine
impl Node<(), Payload, GeneratedPayload> for BroadcastNode {
    fn from_init(
        _state: (),
        init: InitNodes,
        _sender: mpsc::Sender<Event<Payload, GeneratedPayload>>,
    ) -> anyhow::Result<Self> {
        std::thread::spawn(move || loop {
            let propogation_delay = 450;
            std::thread::sleep(Duration::from_millis(propogation_delay));
        });
        return Ok(BroadcastNode {
            node_id: init.node_id,
            local_id: 1,
            messages: HashSet::new(),
            neighbors: HashSet::new(),
            known_by_node: init
                .node_ids
                .into_iter()
                .map(|node_id| (node_id, HashSet::new()))
                .collect(),
        });
    }

    fn step(
        &mut self,
        event: Event<Payload, GeneratedPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match event {
            | Event::EndOfMessages => {
                // IMPORTANT: handle terminating of Propogate loop
                // NOTE: currently just ends on test end. That is fine
            },
            | Event::GeneratedEvent(message) => {
                let mut reply = message.into_reply(Some(&mut self.local_id));
                match reply.body.payload {
                    | GeneratedPayload::Share { messages } => {
                        for node_to_message in &self.neighbors {
                            let messages_to_send: HashSet<usize> = self
                                .messages
                                .iter()
                                .copied()
                                .filter(|message| {
                                    !self.known_by_node[node_to_message].contains(message)
                                })
                                .collect();

                            // IMPORTANT: For efficiency, only share if there is something to share.
                            if messages_to_send.len() != 0 {
                                Message {
                                    src: self.node_id.clone(),
                                    dest: node_to_message.clone(),
                                    body: Body {
                                        id: None,
                                        in_reply_to: None,
                                        payload: GeneratedPayload::Share {
                                            messages: messages_to_send,
                                        },
                                    },
                                }
                                .send(&mut *output, "Propogate")
                                .context(format!(
                                    "Sharing/sending messages to {}",
                                    node_to_message
                                ))?;
                            }
                        }
                    },
                    | GeneratedPayload::ShareOk { messages: values } => {
                        // NOTE: The node knows that the source node has recieved our sent values
                        self.known_by_node
                            .get_mut(&reply.dest) // NOTE: destination of reply
                            .unwrap()
                            .extend(values.iter().copied());
                    },
                }
            },
            | Event::Message(message) => {
                let mut reply = message.into_reply(Some(&mut self.local_id));
                match reply.body.payload {
                    // NOTE: can make this more efficient by sending known_to and updating between
                    // all nodes NOT just within a node
                    | Payload::Echo { echo } => {
                        reply.body.payload = Payload::EchoOk { echo };
                        reply.send(output, "echo")?;
                    },
                    | Payload::Generate => {
                        let guid = Uuid::now_v1(&[self.local_id as u8; 6]);
                        reply.body.payload = Payload::GenerateOk { guid };
                        reply.send(output, "generate")?;
                    },
                    | Payload::Broadcast { message } => {
                        reply.body.payload = Payload::BroadcastOk;
                        self.messages.insert(message);
                        reply.send(output, "broadcast")?;
                    },
                    | Payload::Read => {
                        reply.body.payload = Payload::ReadOk {
                            messages: self.messages.clone(),
                        };
                        reply.send(output, "read")?;
                    },
                    | Payload::Topology { topology } => {
                        reply.body.payload = Payload::TopologyOk;
                        let num_nodes = topology.len();
                        self.neighbors = generate_small_world_toplogy(num_nodes, 4, 0.3)
                            .remove(&self.node_id)
                            .unwrap();
                        reply.send(output, "topology")?;
                    },
                    | Payload::EchoOk { .. }
                    | Payload::GenerateOk { .. }
                    | Payload::BroadcastOk
                    | Payload::ReadOk { .. }
                    | Payload::TopologyOk => {},
                }
                self.local_id += 1;
            },
        }
        return Ok(());
    }
}

fn main() -> anyhow::Result<()> {
    return event_loop::<BroadcastNode, _, _,_>(());
}
