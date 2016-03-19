require 'protobuf/message'

module Comm
  module Messages
    class ChatPayload < Protobuf::Message
      required :string, :sender, 1
      required :string, :text, 3
      required :int64, :timestamp, 4
    end

    class Chat < Protobuf::Message
      required :string, :address, 1
      required :string, :recipient, 2
      required :string, :payload, 3

      def hash
        [self.class, address.hash].hash
      end

      def eql?(other)
        self.class == other.class && self == other
      end

      def ==(other)
        address == other.address
      end
    end

    class Peer < Protobuf::Message
      required :string, :address, 1
      required :string, :host, 2
      required :int32, :port, 3
    end

    class Message < Protobuf::Message
      optional Messages::Peer, :peer, 1
      optional Messages::Chat, :chat, 2

      def unwrap
        peer or chat
      end
    end

    def self.decode(encoded_message)
      Message.decode(encoded_message)
    end

    def self.decode_from(io)
      Message.decode_from(io)
    end

    def self.encode(message)
      case message
      when Peer
        Message.new(peer: message)
      when Chat
        Message.new(chat: message)
      end.encode
    end
  end
end
