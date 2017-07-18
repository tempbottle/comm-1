# Comm Protocol

## Network

The comm network is made up of nodes. Each node has an address. The distance
between two nodes is the XOR of their addresses. Each node maintains a routing
table with a set of peers based on their distance from itself.

The routing table contains up to 160 node buckets. Each one contains `k` (where
`k` is 8 by default) peer nodes, and they form "rings" at increasing distances
from node itself.

A node can deliver a packet to any other node to whom it has a connection and
whose address it knows.

In the implementation, a node maintains all this network state via the
`Network` struct.

## Messaging

Messages can be delivered between nodes regardless of whether they have a direct
connection (having each other in their routing table) by way of relaying.

When a node wants to deliver a message to another node, it sends it to the `k`
nodes in its routing table nearest to the recipient.

Whenever a node receives a message for which it is NOT the recipient, it relays
it along to the `k` nearest nodes to the recipient that it knows about.

In effect, this causes a flood of relay traffic, but because of the network
structure imposed by each node keeping buckets of nodes at varying distances,
the message travels in the direction of the recipient, and doesn't flood the
network as a whole.

Another feature of message delivery is that messages can be stored in the network
until the recipient is available to receive it. Nodes SHOULD relay a message
repeatedly, but at longer and longer intervals using exponential backoff.

When the recipient finally receives the message, it MUST send an
acknowledgement back to the sender using the same delivery procedure: relaying.
The intent is for all intermediary nodes that previously relayed the message to
now see the acknowledgement. The sender should permanently store this
acknowledgement with the message so that it may be able to distribute it
whenever another node tries to send it the (already received) message.

If a node receives an acknowledgement, it should relay it towards the recipient and 
temporarily store it. If the node has the message to which the acknowledgement
pertains to queued to send later (because of repeated relaying), it MUST cancel
the relaying of said message and MUST NOT relay it again ever.

If a node receives a message from another node, and it has an acknowledgement
for the same message, it MUST relay the acknowledgement to the node that just
sent them the message. It should otherwise ignore the message and MUST NOT
relay it.

In order to fully distribute the acknowledgement of a message to all nodes who
may still be relaying the message, a node SHOULD hang onto an acknowledgement
for a reasonable length of time (this time may reset every time it
re-encounters the original message and thus needs to relay the acknowledgement).
