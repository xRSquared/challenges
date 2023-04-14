use rust_distributed_sys_challenge::*;

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
enum PayLoad {
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
    Share {
        messages: HashSet<usize>,
    },
    ShareOk {
        messages: HashSet<usize>,
    },
}

struct DistributedNode {
    node_id: String,
    local_id: usize,
    messages: HashSet<usize>,
    neighborhood: HashSet<String>,
    known_by_node: HashMap<String, HashSet<usize>>,
}

// NOTE: state machine
impl Node<(), PayLoad> for DistributedNode {
    fn from_init(
        _state: (),
        init: InitNodes,
        sender: mpsc::Sender<Event<PayLoad>>,
    ) -> anyhow::Result<Self> {
        std::thread::spawn(move || loop {
            // 10 -> pmax: 0
            // 25 -> pmax: 20
            // 50 -> pmax: 48
            // 100 -> pmax: 156
            std::thread::sleep(Duration::from_millis(10));
            if let Err(_) = sender.send(Event::Propogate) {
                return Ok(());
            }
        });
        return Ok(DistributedNode {
            node_id: init.node_id,
            local_id: 1,
            messages: HashSet::new(),
            neighborhood: HashSet::new(),
            known_by_node: init
                .node_ids
                .into_iter()
                .map(|node_id| (node_id, HashSet::new()))
                .collect(),
        });
    }

    fn step(&mut self, event: Event<PayLoad>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match event {
            | Event::EndOfMessages => {
                // IMPORTANT: handle terminating of Propogate loop
                // NOTE: currently just ends on test end. That is fine
            },
            | Event::Propogate => {
                for node_to_message in &self.neighborhood {
                    let messages_to_send: HashSet<usize> = self
                        .messages
                        .iter()
                        .copied()
                        .filter(|message| !self.known_by_node[node_to_message].contains(message))
                        .collect();

                    // IMPORTANT: only share if there is something to share
                    if messages_to_send.len() != 0 {
                        eprintln!(
                            "notifing {}/{}",
                            messages_to_send.len(),
                            self.messages.len()
                        );
                        Message {
                            src: self.node_id.clone(),
                            dest: node_to_message.clone(),
                            body: Body {
                                id: None,
                                in_reply_to: None,
                                payload: PayLoad::Share {
                                    messages: messages_to_send,
                                },
                            },
                        }
                        .send(&mut *output, "Propogate")
                        .context(format!("sharing messages to {}", node_to_message))?;
                    }
                }
            },
            | Event::Message(message) => {
                let mut reply = message.into_reply(Some(&mut self.local_id));
                match reply.body.payload {
                    // NOTE: can make this more efficient by sending known_to and updating between
                    // all nodes NOT just within a node
                    // IMPORTANT: if that isn't enough to pass challenge 3d then change the network
                    // topology
                    | PayLoad::Share { messages: values } => {
                        // NOTE: The Node knows that source node knows that values that the source node
                        // sent
                        self.known_by_node
                            .get_mut(&reply.dest) // NOTE: destination of reply
                            .unwrap()
                            .extend(values.iter().copied());
                        self.messages.extend(&values);
                        reply.body.payload = PayLoad::ShareOk { messages: values };
                        reply.send(output, "Share")?;
                    },
                    | PayLoad::ShareOk { messages: values } => {
                        // NOTE: The node knows that the source node has recieved our sent values
                        self.known_by_node
                            .get_mut(&reply.dest) // NOTE: destination of reply
                            .unwrap()
                            .extend(values.iter().copied());
                    },
                    | PayLoad::Echo { echo } => {
                        reply.body.payload = PayLoad::EchoOk { echo };
                        reply.send(output, "echo")?;
                    },
                    | PayLoad::Generate => {
                        let guid = Uuid::now_v1(&[self.local_id as u8; 6]);
                        reply.body.payload = PayLoad::GenerateOk { guid };
                        reply.send(output, "generate")?;
                    },
                    | PayLoad::Broadcast { message } => {
                        reply.body.payload = PayLoad::BroadcastOk;
                        self.messages.insert(message);
                        reply.send(output, "broadcast")?;
                    },
                    | PayLoad::Read => {
                        reply.body.payload = PayLoad::ReadOk {
                            messages: self.messages.clone(),
                        };
                        reply.send(output, "read")?;
                    },
                    | PayLoad::Topology { mut topology } => {
                        reply.body.payload = PayLoad::TopologyOk;
                        self.neighborhood = topology.remove(&self.node_id).unwrap();
                        reply.send(output, "topology")?;
                    },
                    | PayLoad::EchoOk { .. }
                    | PayLoad::GenerateOk { .. }
                    | PayLoad::BroadcastOk
                    | PayLoad::ReadOk { .. }
                    | PayLoad::TopologyOk => {},
                }
                self.local_id += 1;
            },
        }
        return Ok(());
    }
}

fn main() -> anyhow::Result<()> {
    return event_loop::<DistributedNode, _, _>(());
}
