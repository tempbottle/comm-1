require 'celluloid/current'
require 'celluloid/io'
require 'securerandom'
require 'comm/messages'
require 'comm/peer'
require 'comm/version'

module Comm
  class Node
    include Celluloid::IO

    def initialize(host, port, address: SecureRandom.uuid)
      @address = address
      @host = host
      @port = port
      @server = TCPServer.new(host, port)
      @peers = {}
    end

    def accept_connections
      puts "Accepting connections as #{@address}"
      loop { async.establish_peer(@server.accept) }
    end

    def connect_to(host, port)
      puts "-> Trying to connect to #{host}:port"
      socket = TCPSocket.open(host, port)
      async.establish_peer(socket)
    end

    def broadcast(message)
      @peers.each do |address, peer|
        peer.send(message)
      end
    end

    private

    def add_peer(peer)
      return if @peers.has_key?(peer.address)
      puts "-> Adding peer #{peer.inspect}"
      @peers[peer.address] = peer
      async.listen_to(peer)
      async.announce_peer(peer)
    end

    def announce_peer(peer_to_announce)
      @peers.each do |address, peer|
        next if peer == peer_to_announce
        puts "-> Announcing #{peer_to_announce.inspect} to #{peer.inspect}"
        peer.send(Messages.encode(peer_to_announce.announcement))
      end
    end

    def drop_peer(peer)
      puts "-> Dropping peer #{peer.inspect}"
      peer.disconnect
      @peers.delete(peer.address)
    end

    def establish_peer(socket)
      message = Messages.encode(Messages::Synchronize.new(
        address: @address,
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
        puts "-> Message from #{peer.inspect}: #{message.inspect}"
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
