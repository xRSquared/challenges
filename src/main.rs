use rust_distributed_sys_challenge::*;

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
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
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<usize>>,
    },
    TopologyOk,
}

struct DistributedNode {
    node: Option<String>,
    local_id: u8,
    messages: Vec<usize>,
    topology: Option<HashMap<String, Vec<usize>>>,
}

// NOTE: state machine
impl Node<(), PayLoad> for DistributedNode {
    fn from_init(_state: (), init: InitNodes) -> anyhow::Result<Self> {
        return Ok(DistributedNode {
            node: init.node_id,
            local_id: 1,
            messages: Vec::new(),
            topology: None,
        });
    }

    fn step(&mut self, input: Message<PayLoad>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            | PayLoad::Echo { echo } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.local_id),
                        in_reply_to: input.body.id,
                        payload: PayLoad::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Serialize response to init")?;
                output.write_all(b"\n").context("Write trailing newline")?;
                self.local_id += 1;
            },
            | PayLoad::EchoOk { .. } => {},
            | PayLoad::Generate => {
                let guid = Uuid::now_v1(&[self.local_id; 6]);
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.local_id),
                        in_reply_to: input.body.id,
                        payload: PayLoad::GenerateOk { guid },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Serialize response to generate")?;
                output.write_all(b"\n").context("Write trailing newline")?;
                self.local_id += 1;
            },
            | PayLoad::GenerateOk { .. } => {},
            | PayLoad::Broadcast { message } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.local_id),
                        in_reply_to: input.body.id,
                        payload: PayLoad::BroadcastOk,
                    },
                };
                self.messages.push(message);
                serde_json::to_writer(&mut *output, &reply)
                    .context("Serialize response to broadcast.")?;
                output.write_all(b"\n").context("Write trailing newline.")?;
                self.local_id += 1;
            },
            | PayLoad::BroadcastOk => {},
            | PayLoad::Read => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.local_id),
                        in_reply_to: input.body.id,
                        payload: PayLoad::ReadOk {
                            messages: self.messages.clone(),
                        },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Serialize response to read.")?;
                output.write_all(b"\n").context("Write trailing newline.")?;
                self.local_id += 1;
            },
            | PayLoad::ReadOk { .. } => {},
            | PayLoad::Topology { topology } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.local_id),
                        in_reply_to: input.body.id,
                        payload: PayLoad::TopologyOk,
                    },
                };
                self.topology = Some(topology);
                serde_json::to_writer(&mut *output, &reply)
                    .context("Serialize response to topology.")?;
                output.write_all(b"\n").context("Write trailing newline.")?;
                self.local_id += 1;
            },
            | PayLoad::TopologyOk => {},
        }
        return Ok(());
    }
}

fn main() -> anyhow::Result<()> {
    return event_loop::<DistributedNode, _, _>(());
}
