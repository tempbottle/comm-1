require 'protobuf/message'

module Comm
  module Messages
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
    end

    def self.decode(encoded_message)
      message = Message.decode(encoded_message)
      if message.synchronize
        message.synchronize
      elsif message.peer
        message.peer
      end
    end

    def self.encode(message)
      case message
      when Peer
        Message.new(peer: message)
      when Synchronize
        Message.new(synchronize: message)
      end.encode
    end
  end
end
