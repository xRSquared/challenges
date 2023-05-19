// Purpose: Broadcast messages to all nodes in the network.
use anyhow::{Context, Ok};
use rust_distributed_sys_challenge::*;
use serde::{Deserialize, Serialize};
use std::{io::StdoutLock, sync::mpsc};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")] // IMPORTANT: returns {type:"echo", echo:"..."}
#[serde(rename_all = "snake_case")]
enum PayLoad {
    Add { delta: usize },
    AddOk,
    Read,
    ReadOk { value: usize },
}

struct GlobalCounterNode {
    node_id: String,
    local_id: usize,
    global_value: usize,
}

// NOTE: state machine
impl Node<(), PayLoad, ()> for GlobalCounterNode {
    fn from_init(
        _state: (),
        init: InitNodes,
        _sender: mpsc::Sender<Event<PayLoad, ()>>,
    ) -> anyhow::Result<Self> {
        return Ok(GlobalCounterNode {
            node_id: init.node_id,
            local_id: 1,
            global_value: 1,
        });
    }

    fn step(&mut self, event: Event<PayLoad, ()>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match event {
            | Event::EndOfMessages => {
                // IMPORTANT: handle terminating of Propogate loop
                // NOTE: currently just ends on test end. That is fine
            },
            | Event::GeneratedEvent(_) => {},
            | Event::Message(message) => {
                let mut reply = message.into_reply(Some(&mut self.local_id));
                match reply.body.payload {
                    | PayLoad::Read => {
                        reply.body.payload = PayLoad::ReadOk {
                            value: self.global_value,
                        };
                        reply.send(output, "read")?;
                    },
                    | PayLoad::Add { delta } => {
                        self.global_value += delta;
                        reply.body.payload = PayLoad::AddOk;
                        reply.send(output, "add")?;
                    },
                    | PayLoad::ReadOk { .. } | PayLoad::AddOk => {},
                }
                self.local_id += 1;
            },
        }
        return Ok(());
    }
}

fn main() -> anyhow::Result<()> {
    return event_loop::<GlobalCounterNode, _, _, _>(());
}
