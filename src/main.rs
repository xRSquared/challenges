use std::io::{StdoutLock, Write};

use anyhow::{bail, Context, Ok};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}
#[derive(Debug, Serialize, Deserialize)]
struct Body {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)] // IMPORTANT: removes payload from json serialization
    pub payload: PayLoad,
}

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

struct Node {
    id: usize,
}

// NOTE: state machine
impl Node {
    pub fn step(&mut self, input: Message, output: &mut StdoutLock) -> anyhow::Result<()> {
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
    let stdin = std::io::stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = std::io::stdout().lock();

    let mut state = Node { id: 0 };

    for input in inputs {
        let input = input.context("Maelstrom input could not be deserialized")?;
        state
            .step(input, &mut stdout)
            .context("Node step function failed")?;
    }
    return Ok(());
}
