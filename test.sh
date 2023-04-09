#!/usr/bin/env fish
set test $argv;

if test "$test" = "echo"
    ~/maelstrom/maelstrom test -w echo --bin ./target/debug/rust-distributed-sys-challenge --node-count 1 --time-limit 10
end
if test "$test" = "unique-ids"
    ~/maelstrom/maelstrom test -w unique-ids --bin ./target/debug/rust-distributed-sys-challenge --node-count 1 --time-limit 10
end
