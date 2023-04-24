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

- Solution to \[3b\](#3b: multi-node broadcast) is already fault tolerant

### 3d: Efficient Broadcast, Part I

#### Problem

- We want to make the Propagation of values efficient with the following properties:
  - Messages-per-operation is below 30
  - Median latency is below 400ms
  - Maximum latency is below 600ms

#### Solution

**Improve efficency of operations:**

1. Move from using Vec to using HashSet, which allows for faster lookups
2. During Propogation, only send values that haven't been sent to given node before
   (reduced multi-broadcast maximum by 100ms)
   - To preserve fault tolerance, require confirmation before adding a value
     `known_by_node` list
3. Only send a `Payload::Share` when there is something to share
   (reduced multi-broadcast maximum by 20ms)
4. reduced sleep duration to `10ms` in Propogation event spawner
   (reduced multi-broadcast maximum to 5ms)

After implementing all of these micro/macro optimizations
it still wasn't enough to meet the goals. The only other improvement that
I can see is changing/optimizing the topology provided by Maelstrom.

**Improve Topology - send information efficiently**

- I implemented a [small world topology](https://en.wikipedia.org/wiki/Small-world_network)
  using the [Watts-Strogatz model](https://en.wikipedia.org/wiki/Watts-Strogatz_model)

> The key idea is that we want to have a small world topology
> where nodes are connected to their neighbors and a random set of other nodes.

The key parameters are:

- local_cluster_count: 4
- rewire_probability: 0.3
- propoganation_delay: 250ms

With the values above, I was able to achive all the performance goals.

> We can improve the performance even more by batching the propogation events

### 3e: Efficient Broadcast, Part 2

> Easier than expected

#### Problem

- We want to make the Propagation of values efficient with the following properties:
  - Messages-per-operation is below 20
  - Median latency is below 1 second
  - Maximum latency is below 2 seconds

#### Solution

After implementing the solutions described
in \[3d\](#3d: Efficient Broadcast, Part I)
under `Imporve efficency of operations` all of these efficency goals were met by
setting the delay between Share Events(propagation_delay) to 100ms

Using the small world topology we can achieve the efficency goals
with the following parameters:

- local_cluster_count: 4
- rewire_probability: 0.3
- propoganation_delay: 450ms

## Learnings

- `anyhow` package is great!
  - can provide context to errors

## Notes

use `~/maelstrom/maelstrom serve` to view logs in browser.
