require 'celluloid/current'
require 'celluloid/io'
require 'securerandom'
require 'comm/messages'
require 'comm/address'
require 'comm/peer'
require 'comm/version'
require 'comm/peer_pool'

module Comm
  class Node
    include Celluloid::IO

    attr_reader :address

    def initialize(host, port, secret:)
      @address = Address.for_content(secret)
      @host = host
      @port = port
      @server = TCPServer.new(host, port)
      @peers = PeerPool.new(size: 10)
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
      message = Messages.encode(Messages::Synchronize.new(
        address: @address.to_s,
        host: @host,
        port: @port
      ))
      socket.send(message.encode, 0)
      if message = Messages.decode(socket.recv(4096, 0))
        peer = Peer.new(
          address: message.address,
          host: message.host,
          port: message.port,
          socket: socket
        )
        async.add_peer(peer)
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
        end
      end
    rescue EOFError
      drop_peer(peer)
    end
  end
end
