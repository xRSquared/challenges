# Distributed Systems Challege in Rust

I saw the fly.io [Gossip Gloomers Challenge](https://fly.io/dist-sys/) being
discussed on Hacker News, and I'm teaching myself Rust.
So I decided to take a stab at making an implementation...

## Solutions

### 3b: multi-node broadcast

#### Problem

- We need to send information to other nodes without the node being prompted to
  send the information

  - We could send it after every broadcast but that would be inefficient

- Our solution could be written in one of two ways

  - threading
    - We could spawn another thread that generates a Propogate event on a timer
  - async
    - We could implement a process that checks if there are any new values
      (values that other nodes don't have)
      and sends values to other nodes in their neighborhood
      - To avoid dealing with consensus, we could implement this with a timer
        on a fully connected graph

#### Solution

I decided to solve the Problem by implementing it with two threads.

- One thread deals with Maelstrom Events
- another thread deals with Propogating values on a set timer

> Doing it asynchronously seemed weird to me since the Propogating is
> independent of Maelstrom Events.

> NOTE:  I learned threading from the offical [rust book](https://doc.rust-lang.org/book/ch16-01-threads.html)

## Learnings

- `anyhow` package is great!
  - can provide context to errors
