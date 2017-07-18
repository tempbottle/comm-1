# Comm

A hypothetical, distributed, encrypted, instant messaging protocol.

* Read the [protocol][protocol] for an overview of how comm works.

## Project Goals

* No centralized server
* Messages should be stored in the network for some reasonable amount of time
  until recipient is available
* It should be difficult for an adversary to silence a participant (prevent
  them from sending messages)
* It should be difficult for an adversary to deafen a participant (prevent them
  from receiving messages)

## What the what

1. Client relays a message to the network
2. Network nodes store and re-relay messages they receive
3. Network nodes drop messages that they've relayed for a reasonable amount of
   time, and a reasonable number of times
4. Recipient finally gets message, relays acknowledgement
5. Network nodes drop messages after verifying their acknowledgement
6. Network nodes re-relay acknowledgement to nodes that need to stop relaying
   the message

Please read the [protocol][protocol] for a more detailed overview.

## Development

You need to have the protobuf crate installed and in your path. Add your multirust
crate bin path to your `PATH` if you use multirust:

    cargo install protobuf
    PATH=~/.multirust/toolchains/stable/cargo/bin:$PATH

## Usage

You can fire up a CLI chat client by running

    cargo run comm -- $SECRET 0.0.0.0:$PORT [1.2.3.4:$OTHER_NODE_PORT]

Where SECRET is a word that will be SHA1 hashed into your node's address, PORT
is the local port you want to run on, and then third argument is the address
and port of another node. This other node is a "bootstrap node" and will be
your entrypoint into the network. It can be another `comm` client running
without a bootstrap router of it's own.

An interactive CLI will start, and you can send a message to another node by
entering its address, followed by a message:

    44751799925b964a00bae3863cc4236f9bb8d519 Hi there!

# Similar Projects

* [Briar](https://briarproject.org/)
* [Ensichat](https://github.com/Nutomic/ensichat)
* [Tox](https://tox.chat/)

[protocol]: PROTOCOL.md
