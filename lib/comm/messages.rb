require 'protobuf/message'

module Comm
  module Messages
    class Chat < Protobuf::Message
      required :string, :address, 1
      required :string, :recipient, 2
      required :string, :text, 3

      def hash
        address.hash
      end

      def eql?(other)
        self.class == other.class && self == other
      end

      def ==(other)
        address == other.address
      end
    end

    class Synchronize < Protobuf::Message
      required :string, :address, 1
      required :string, :host, 2
      required :int32, :port, 3
    end

    class Peer < Protobuf::Message
      required :string, :address, 1
      required :string, :host, 2
      required :int32, :port, 3
    end

    class Message < Protobuf::Message
      optional Messages::Synchronize, :synchronize, 1
      optional Messages::Peer, :peer, 2
      optional Messages::Chat, :chat, 3
    end

    def self.decode(encoded_message)
      message = Message.decode(encoded_message)
      message.synchronize or
        message.peer or
        message.chat
    end

    def self.encode(message)
      case message
      when Peer
        Message.new(peer: message)
      when Synchronize
        Message.new(synchronize: message)
      when Chat
        Message.new(chat: message)
      end.encode
    end
  end
end
