use std::io::StdoutLock;

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize, de::DeserializeOwned};


#[derive(Debug, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    pub dest: String,
    pub body: Body<Payload>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)] // IMPORTANT: removes payload from json serialization
    pub payload: Payload,
}

pub trait Node<Payload> {
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn event_loop<S,Payload>(mut state: S) -> anyhow::Result<()>
where
    S: Node<Payload>,
    Payload: DeserializeOwned,
{
    let stdin = std::io::stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<Payload>>();

    let mut stdout = std::io::stdout().lock();


    for input in inputs {
        let input = input.context("Maelstrom input could not be deserialized")?;
        state
            .step(input, &mut stdout)
            .context("Node step function failed")?;
    }
    return Ok(());
}
