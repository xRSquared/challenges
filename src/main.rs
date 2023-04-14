use rust_distributed_sys_challenge::*;

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::StdoutLock, sync::mpsc, time::Duration};
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
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Share {
        messages: Vec<usize>,
    },
}

struct DistributedNode {
    node_id: String,
    local_id: u8,
    messages: Vec<usize>,
    neighborhood: Vec<String>,
}

// NOTE: state machine
impl Node<(), PayLoad> for DistributedNode {
    fn from_init(
        _state: (),
        init: InitNodes,
        sender: mpsc::Sender<Event<PayLoad>>,
    ) -> anyhow::Result<Self> {
        // IMPORTANT: `Propogate` values every 5 seconds
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(5));
            if let Err(_) = sender.send(Event::Propogate) {
                return Ok(());
            }
        });
        return Ok(DistributedNode {
            node_id: init.node_id,
            local_id: 1,
            messages: Vec::new(),
            neighborhood: Vec::new(),
        });
    }

    fn step(&mut self, event: Event<PayLoad>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match event {
            | Event::EndOfMessages => {
                // IMPORTANT: handle terminating of Propogate loop
            },
            | Event::Propogate => {
                for node_to_message in &self.neighborhood {
                    // TODO: remove elements that node n already knows
                    Message {
                        src: self.node_id.clone(),
                        dest: node_to_message.clone(),
                        body: Body {
                            id: None,
                            in_reply_to: None,
                            payload: PayLoad::Share {
                                messages: self.messages.clone(),
                            },
                        },
                    }
                    .send(&mut *output, "Propogate")
                    .context(format!("sharing messages to {}", node_to_message))?;
                }
            },
            // IMPORTANT: Propogate values
            | Event::Message(message) => {
                let mut reply = message.into_reply(Some(&mut self.local_id));
                match reply.body.payload {
                    | PayLoad::Share { messages: values } => {
                        self.messages.extend(&values);
                    },

                    | PayLoad::Echo { echo } => {
                        reply.body.payload = PayLoad::EchoOk { echo };
                        reply.send(output, "echo")?;
                    },
                    | PayLoad::Generate => {
                        let guid = Uuid::now_v1(&[self.local_id; 6]);
                        reply.body.payload = PayLoad::GenerateOk { guid };
                        reply.send(output, "generate")?;
                    },
                    | PayLoad::Broadcast { message } => {
                        reply.body.payload = PayLoad::BroadcastOk;
                        self.messages.push(message);
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
