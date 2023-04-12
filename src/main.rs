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
        let mut reply = input.into_reply(Some(&mut self.local_id));
        let mut reply_to = "";
        match reply.body.payload {
            | PayLoad::Echo { echo } => {
                reply_to = "echo";
                reply.body.payload = PayLoad::EchoOk { echo };
            },
            | PayLoad::Generate => {
                reply_to = "generate";
                let guid = Uuid::now_v1(&[self.local_id; 6]);
                reply.body.payload = PayLoad::GenerateOk { guid };
            },
            | PayLoad::Broadcast { message } => {
                reply_to = "broadcast";
                reply.body.payload = PayLoad::BroadcastOk;
                self.messages.push(message);
                            },
            | PayLoad::Read => {
                reply_to = "read";
                reply.body.payload = PayLoad::ReadOk {
                    messages: self.messages.clone(),
                };
            },
            | PayLoad::Topology { topology } => {
                reply_to = "topology";
                reply.body.payload = PayLoad::TopologyOk;
                self.topology = Some(topology);
            },
            | PayLoad::EchoOk { .. }
            | PayLoad::GenerateOk { .. }
            | PayLoad::BroadcastOk
            | PayLoad::ReadOk { .. }
            | PayLoad::TopologyOk => {},
        }
        serde_json::to_writer(&mut *output, &reply)
            .context(format!("Serialize response to {}", reply_to))?;
        output.write_all(b"\n").context("Write trailing newline")?;
        self.local_id += 1;

        return Ok(());
    }
}

fn main() -> anyhow::Result<()> {
    return event_loop::<DistributedNode, _, _>(());
}
