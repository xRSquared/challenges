#!/usr/bin/env fish
set test $argv

switch "$test"
    case echo
        ~/maelstrom/maelstrom test -w echo --bin ./target/debug/broadcast --node-count 1 --time-limit 10
    case unique-ids
        ~/maelstrom/maelstrom test -w unique-ids --bin ./target/debug/broadcast --node-count 1 --time-limit 10
    case single-broadcast
        ~/maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 1 --time-limit 20 --rate 10
    case multi-broadcast
        ~/maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 5 --time-limit 20 --rate 10
    case fault-tolarant
        ~/maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 5 --time-limit 20 --rate 10 --nemesis partition
    case efficient-broadcast
        ~/maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 25 --time-limit 20 --rate 100 --latency 100
    case g-counter
        ~/maelstrom/maelstrom test -w g-counter --bin ./target/debug/g_counter --node-count 3 --rate 100 --time-limit 20 --nemesis partition
    case serve
        ~/maelstrom/maelstrom serve
    case '*'
        echo unknown test $test
end
