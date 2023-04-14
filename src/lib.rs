use std::io::{BufRead, StdoutLock, Write};
use std::sync::mpsc;
use std::thread;

use anyhow::{Context, Ok};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug)]
pub enum Event<Payload> {
    Message(Message<Payload>),
    Propogate,
    // NOTE: signifies end of stdin messages - used to stop `Propogate` loop
    EndOfMessages,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    pub dest: String,
    pub body: Body<Payload>,
}

impl<Payload> Message<Payload> {
    /// Turn message into a reply
    pub fn into_reply(self, id: Option<&mut u8>) -> Self {
        return Self {
            src: self.dest,
            dest: self.src,
            body: Body {
                id: Some(*id.unwrap()),
                in_reply_to: self.body.id,
                payload: self.body.payload,
            },
        };
    }
    /// Send message to stdin
    pub fn send(self, output: &mut StdoutLock, reply_to: &str) -> anyhow::Result<()>
    where
        Payload: Serialize,
    {
        serde_json::to_writer(&mut *output, &self)
            .context(format!("serialize response to {}", reply_to))?;
        output.write_all(b"\n").context("write trailing newline")?;
        return Ok(());
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<u8>,
    pub in_reply_to: Option<u8>,
    #[serde(flatten)] // IMPORTANT: removes "payload" from json serialization
    pub payload: Payload,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(InitNodes),
    InitOk,
}

//NOTE: needed from Go implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitNodes {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<State, Payload> {
    fn from_init(
        state: State,
        init: InitNodes,
        sender: mpsc::Sender<Event<Payload>>,
    ) -> anyhow::Result<Self>
    //IMPORTANT: need to tell compiler `Node` is of fixed size
    where
        Self: Sized;
    fn step(&mut self, event: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

// TODO: move initialization to private function
pub fn event_loop<N, State, Payload>(inital_state: State) -> anyhow::Result<()>
where
    Payload: DeserializeOwned + Send + 'static,
    N: Node<State, Payload>,
{
    let stdin = std::io::stdin().lock();
    let mut lines = stdin.lines();

    let mut stdout = std::io::stdout().lock();

    let init_message: Message<InitPayload> = serde_json::from_str(
        &lines
            .next()
            .expect("No init message received.")
            .context("Failed to read init message.")?,
    )
    .context("Init message could not be deserialized!")?;

    let InitPayload::Init(init) = init_message.body.payload else {
        panic!("First message should be an init!");
    };

    let (sender, reciever) = mpsc::channel();
    let mut node: N =
        Node::from_init(inital_state, init, sender.clone()).context("Node initilization failed")?;

    // NOTE: can't into_reply() here because `init` was consumed above
    let reply = Message {
        src: init_message.dest,
        dest: init_message.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_message.body.id,
            payload: InitPayload::InitOk,
        },
    };

    serde_json::to_writer(&mut stdout, &reply).context("Serialize response to init.")?;
    stdout
        .write_all(b"\n")
        .context("Writing trailing newline.")?;

    drop(lines);
    // NOTE: Handle message parsing in other thread
    let handler = thread::spawn(move || {
        let stdin = std::io::stdin().lock();
        let lines = stdin.lines();

        for input in lines {
            let input = input.context("Maelstrom input could not be read.")?;
            let message: Message<Payload> = serde_json::from_str(&input)
                .context("Maelstrom input could not be deserialized.")?;
            if let Err(_) = sender.send(Event::Message(message)) {
                return Ok(());
            }
        }
        if let Err(_) = sender.send(Event::EndOfMessages) {
            return Ok(());
        }
        return Ok(());
    });

    for message in reciever {
        node.step(message, &mut stdout)
            .context("Node step function failed.")?;
    }
    handler.join().expect("thread paniced")?;
    return Ok(());
}
