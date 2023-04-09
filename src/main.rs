use rust_distributed_sys_challenge::*;

use anyhow::{bail, Context, Ok};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")] // IMPORTANT: returns {type:"echo", echo:"..."}
#[serde(rename_all = "snake_case")]
enum PayLoad {
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    // IMPORTANT: these cases need to be handled to run test suite
    // NOTE: these have to be handled because we aren't using the go library from tutorial
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

struct DistributedNode {
    id: usize,
}

// NOTE: state machine
impl Node<PayLoad> for DistributedNode {
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

            | PayLoad::Init { .. } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: PayLoad::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("Serialize response to init")?;
                output.write_all(b"\n").context("Write trailing newline")?;
                self.id += 1;
            },

            // NOTE: this should only be used by Maelstrom client
            | PayLoad::InitOk { .. } => bail!("Recieved init_ok message. This shouldn't happen"),
        }
        return Ok(());
    }
}

fn main() -> anyhow::Result<()> {
    event_loop(DistributedNode { id: 0 })
}
