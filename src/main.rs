use rust_distributed_sys_challenge::*;

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};
use uuid::{self, Uuid};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")] // IMPORTANT: returns {type:"echo", echo:"..."}
#[serde(rename_all = "snake_case")]
enum PayLoad {
    //NOTE: find a way to remoke OKs from this enum
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
}

struct DistributedNode {
    node: Option<String>,
    id: u8,
}

// NOTE: state machine
impl Node<(), PayLoad> for DistributedNode {
    fn from_init(_state: (), init: InitNodes) -> anyhow::Result<Self> {
        return Ok(DistributedNode {
            node: init.node_id,
            id: 1,
        });
    }

    fn step(&mut self, input: Message<PayLoad>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            | PayLoad::Echo { echo } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: PayLoad::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Serialize response to init")?;
                output.write_all(b"\n").context("Write trailing newline")?;
                self.id += 1;
            },
            | PayLoad::EchoOk { .. } => {},
            | PayLoad::Generate => {
                let guid = Uuid::now_v1(&[self.id; 6]);
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: PayLoad::GenerateOk { guid },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to generate")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            },
            | PayLoad::GenerateOk { .. } => {},
        }
        return Ok(());
    }
}

fn main() -> anyhow::Result<()> {
    return event_loop::<DistributedNode, _, _>(());
}
