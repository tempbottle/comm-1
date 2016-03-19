require 'celluloid/current'
require 'celluloid/io'
require 'logger'
require 'openssl'
require 'securerandom'
require 'comm/address'
require 'comm/cli_client'
require 'comm/message_relay'
require 'comm/message_registry'
require 'comm/messages'
require 'comm/null_client'
require 'comm/peer'
require 'comm/peer_pool'
require 'comm/version'

module Comm
  class Node
    include Celluloid::IO
    include Celluloid::Internals::Logger

    attr_reader :address

    def initialize(host, port, key:, logger: ::Logger.new('comm.log'))
      @address = Address.for_content(key.public_key.export)
      @host = host
      @port = port

      @client = NullClient.new
      @server = TCPServer.new(host, port)
      @peers = PeerPool.new(size: 10)
      @message_relay = MessageRelay.new(self)
      @messages = MessageRegistry.new

      Celluloid.logger = logger
    end

    def attach_client(client)
      @client = client
    end

    def run
      async.accept_connections
      message_relay.async.run
    end

    def stop
      message_relay.stop
      info 'Stopping node'
    end

    def deliver_chat(text, to: recipient)
      payload = Messages::ChatPayload.new(
        sender: address.to_s,
        text: text,
        timestamp: Time.now.to_i).encode
      message = Messages::Chat.new(
        address: Address.for_content(payload).to_s,
        recipient: to.address.to_s,
        payload: payload)
      async.broadcast(message)
    end

    def accept_connections
      info "-> Accepting connections as #{@address}"
      loop { async.handle_connection(@server.accept) }
    end

    def connect_to(host, port)
      info "-> Trying to connect to #{host}:#{port}"
      TCPSocket.open(host, port) do |socket|
        introduction = Messages.encode(Messages::Peer.new(
          address: @address.to_s,
          host: @host,
          port: @port
        ))
        socket.send(introduction.encode, 0)
      end
    end

    def broadcast(message)
      message = Messages.encode(message)

      peers.each do |peer|
        peer.send(message)
      end
    end

    private

    attr_reader :client, :peers, :message_relay, :messages

    def add_peer(peer)
      peers.add(peer) do
        info "-> Adding peer #{peer.inspect}"
        async.announce_peer(peer)
        client.add_peer(peer)
        async.connect_to(peer.host, peer.port);
      end
    end

    def announce_peer(peer_to_announce)
      peers.each do |peer|
        next if peer == peer_to_announce
        info "-> Announcing #{peer_to_announce.inspect} to #{peer.inspect}"
        peer.send(Messages.encode(peer_to_announce.announcement))
      end
    end

    def handle_connection(socket)
      message = Messages.decode_from(socket.to_io).unwrap
      info "-> Recv message: #{message.inspect}"

      case message
      when Messages::Peer
        peer = Peer.new(
          address: Address.new(message.address),
          host: message.host,
          port: message.port)
        async.add_peer(peer)
      when Messages::Chat
        if Address.new(message.recipient) == address
          messages.add(message) do
            payload = Messages::ChatPayload.decode(message.payload)
            client.add_message(payload)
          end
        else
          message_relay.add message
        end
      end

    rescue EOFError
    ensure
      socket.close
    end
  end
end
