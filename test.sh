#!/usr/bin/env fish
set test $argv;

switch "$test"
    case "echo"
        ~/maelstrom/maelstrom test -w echo --bin ./target/debug/rust-distributed-sys-challenge --node-count 1 --time-limit 10
    case "unique-ids"
        ~/maelstrom/maelstrom test -w unique-ids --bin ./target/debug/rust-distributed-sys-challenge --node-count 1 --time-limit 10
    case "single-broadcast"
        ~/maelstrom/maelstrom test -w broadcast --bin ./target/debug/rust-distributed-sys-challenge --node-count 1 --time-limit 20 --rate 10
    case "multi-broadcast"
        ~/maelstrom/maelstrom test -w broadcast --bin ./target/debug/rust-distributed-sys-challenge --node-count 5 --time-limit 20 --rate 10
    case "fault-tolarant"
        ~/maelstrom/maelstrom test -w broadcast --bin ./target/debug/rust-distributed-sys-challenge --node-count 5 --time-limit 20 --rate 10 --nemesis partition
    case '*'
        echo unknown test $test
end
