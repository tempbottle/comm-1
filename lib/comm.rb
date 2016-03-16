require 'celluloid/current'
require 'celluloid/io'
require 'securerandom'
require 'comm/messages'
require 'comm/message_relay'
require 'comm/address'
require 'comm/peer'
require 'comm/version'
require 'comm/peer_pool'

module Comm
  class Node
    include Celluloid::IO
    include Celluloid::Internals::Logger

    attr_reader :address

    def initialize(host, port, secret:)
      @address = Address.for_content(secret)
      @host = host
      @port = port
      @server = TCPServer.new(host, port)
      @peers = PeerPool.new(size: 10)
      @message_relay = MessageRelay.new(self)
    end

    def run
      async.accept_connections
      @message_relay.async.run
    end

    def accept_connections
      info "-> Accepting connections as #{@address}"
      loop { async.establish_peer(@server.accept) }
    end

    def connect_to(host, port)
      info "-> Trying to connect to #{host}:#{port}"
      socket = TCPSocket.open(host, port)
      async.establish_peer(socket)
    end

    def broadcast(message)
      message = Messages.encode(message)

      peers.each do |peer|
        peer.send(message)
      end
    end

    private

    attr_reader :peers

    def add_peer(peer)
      peers.add(peer) do
        async.listen_to(peer)
        async.announce_peer(peer)
      end
    end

    def announce_peer(peer_to_announce)
      peers.each do |peer|
        next if peer == peer_to_announce
        info "-> Announcing #{peer_to_announce.inspect} to #{peer.inspect}"
        peer.send(Messages.encode(peer_to_announce.announcement))
      end
    end

    def drop_peer(peer)
      info "-> Dropping peer #{peer.inspect}"
      peer.disconnect
      peers.remove(peer)
    end

    def establish_peer(socket)
      introduction = Messages.encode(Messages::Peer.new(
        address: @address.to_s,
        host: @host,
        port: @port
      ))
      socket.send(introduction.encode, 0)

      loop do
        response = Messages.decode(socket.recv(4096, 0))
        case response
        when Messages::Peer
          peer = Peer.new(
            address: Address.new(response.address),
            host: response.host,
            port: response.port,
            socket: socket
          )
          async.add_peer(peer)
          break
        end
      end
    rescue EOFError
      socket.close
    end

    def listen_to(peer)
      loop do
        message = Messages.decode(peer.recv(4096))
        case message
        when Messages::Peer
          connect_to(message.host, message.port)
        when Messages::Chat
          if Address.new(message.recipient) == address
            puts "<#{peer.address}> #{message.text}"
          else
            @message_relay.add message
          end
        end
      end
    rescue EOFError
      drop_peer(peer)
    end
  end
end
