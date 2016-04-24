# Comm

A hypothetical, distributed, encrypted, instant messaging protocol.

## Requirements

* No centralized server
* Messages should be stored in the network for some reasonable amount of time until recipient is available

## What the what

0. Client announces a message to the network
  - message has a unique id
  - message has a recipient address (public key)
  - message has a "message" that only recipient can read (using their private key)
0. Network nodes store and re-announce messages they receive
0. Network nodes drop messages that they've re-announced for a reasonable amount of time, and a reasonable number of times
0. Recipient finally gets message, announces acknowledgement
  - acknowledgement has message's unique id
  - acknowledgement has signature of message recipient (using their private key)
0. Network nodes drop messages after verifying their acknowledgement
0. Network nodes re-announce acknowledgement a reasonable number of times

## Development

You need to have the protobuf crate installed and in your path. Add your multirust
crate bin path to your `PATH` if you use multirust:

    cargo install protobuf
    PATH=~/.multirust/toolchains/stable/cargo/bin:$PATH
