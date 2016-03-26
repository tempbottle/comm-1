module Comm
  class Node
    include Celluloid::IO
    include Celluloid::Internals::Logger

    attr_reader :address, :logger, :degree

    def initialize(host, port, key:, logger: nil, degree: 10)
      @address = Address.for_content(key.public_key.export)
      @host = host
      @port = port
      @degree = degree

      @client = NullClient.new
      @server = TCPServer.new(host, port)
      @peers = PeerPool.new(self)
      @message_relay = MessageRelay.new(self)
      @messages = MessageRegistry.new
      @stopped = false

      @logger = Celluloid.logger = logger || ::Logger.new("comm-#{address.to_s}.log")
    end

    def attach_client(client)
      @client = client
    end

    def self_peer
      @self_peer ||= Peer.new(address: address, host: @host, port: @port)
    end

    def run
      async.accept_connections
      message_relay.async.run
    end

    def stop
      info 'Stopping node'
      @stopped = true
      @server.close
      message_relay.stop
    end

    def deliver_chat(text, to: recipient)
      recipient = Comm::Address(to)
      payload = Messages::ChatPayload.new(
        sender: address.to_s,
        text: text,
        timestamp: Time.now.to_i).encode
      message = Messages::Chat.new(
        address: Address.for_content(payload).to_s,
        recipient: recipient.to_s,
        payload: payload)

      message_relay.add(message)
    end

    def accept_connections
      info "-> Accepting connections as #{@address}"
      loop { async.handle_connection(@server.accept) }
    rescue IOError
      info "Server closed"
    end

    def connect_to(host, port)
      info "-> Trying to connect to #{host}:#{port}"
      TCPSocket.open(host, port) do |socket|
        find_peer = Messages.encode(Messages::FindPeer.new(
          requester: Messages::Peer.new(
            address: @address.to_s,
            host: @host,
            port: @port),
          address: @address.to_s))
        socket.send(find_peer, 0)
      end
    rescue Errno::ECONNREFUSED, Errno::ETIMEDOUT
    end

    def broadcast(message)
      encoded_message = Messages.encode(message)

      peers.each do |peer|
        peer.send(encoded_message)
      end
    end

    private

    attr_reader :client, :peers, :message_relay, :messages

    def add_peer(peer)
      return if @stopped
      peers.add(peer) do
        info "-> Adding peer #{peer.inspect}"
        client.update_peers(peers)
        #async.connect_to(peer.host, peer.port);
      end
    end

    def handle_connection(socket)
      return if @stopped

      message = Messages.decode_from(socket.to_io).unwrap
      case message
      when Messages::Peers
        message.peers.each do |announcement|
          peer = Peer.new(
            address: Address.new(announcement.address),
            host: announcement.host,
            port: announcement.port)
          add_peer(peer)
        end
      when Messages::FindPeer
        requester = message.requester
        requester = Peer.new(
          address: Address.new(requester.address),
          host: requester.host,
          port: requester.port)
        nearest = peers.nearest_to(message.address).take(degree).map(&:serialize)
        nearest = Messages.encode(Messages::Peers.new(peers: nearest))
        add_peer(requester)
        requester.send(nearest)
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

    rescue EOFError, Protobuf::InvalidWireType
    ensure
      socket.close
    end
  end
end
