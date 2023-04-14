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

### 3c: Fault Tolerant Broadcast

#### Problem

- Sometimes a node might not be able to communicate with other nodes,
but we must ensure that information passes along to all nodes

#### Solution

- Solution to [3b](#3b: multi-node broadcast) is already fault tolerant

### 3d: Efficient Broadcast, Part I

#### Problem

- We want to make the Propagation of values efficient with the following properties:
    - Messages-per-operation is below 30
    - Median latency is below 400ms
    - Maximum latency is below 600ms

#### Solution

1. Move from using Vec to using HashSet, which allows for faster lookups
2. During Propogation, only send values that haven't been sent to given node before
(reduced multi-broadcast maximum by 100ms)
    - To preserve fault tolerance, require confirmation before adding a value
    `known_by_node` list
3. Only send a `Payload::Share` when there is something to share
(reduced multi-broadcast maximum by 20ms)
4. reduced sleep duration to `10ms` in Propogation event spawner
(reduced multi-broadcast maximum to 5ms)

#### TODO
- play with graph topology to get the pmax below 600ms, currently at 800ms-ish

## Learnings

- `anyhow` package is great!
  - can provide context to errors

## Notes

use `~/maelstrom/maelstrom serve` to view logs in browser.
