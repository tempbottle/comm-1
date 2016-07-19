# Comm Protocol

# Network
## Nodes
## Messages
## Routing
### Node Buckets
### Routing Table

# Messaging
## Clients
## Messages
### TextMessage

    required string id = 1;
    required string sender = 2;
    required string text = 3;

A simple text message produced when a Client sends a message to another Client.

Upon receiving a TextMessage addressed to them, the Client MUST create and
permanently store a MessageAcknowledgement, and deliver it to the sender.

If the receiving Client has a MessageAcknowledgement stored for the received
TextMessage, it MUST forward the MessageAcknowledgement to the Client that
relayed them the TextMessage.

Upon receiving a TextMessage addressed to another Client, if the receiving
client has no MessageAcknowledgement for the TextMessage, the Client MUST
periodically forward it until receiving a MessageAcknowledgement for the
TextMessage.

### MessageAcknowledgement

    required string message_id = 1;

Upon receiving a MessageAcknowledgement, a node MUST store the
MessageAcknowledgement for a reasonable amount of time, and stop forwarding the
message that it pertains to.

If the MessageAcknowledgement is destined for another Client, the receiving
Client MUST forward the MessageAcknowledgement to its destination.
