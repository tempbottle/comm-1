module Comm
  class Peer
    attr_reader :address, :host, :port

    def initialize(address:, host:, port:, socket:)
      @address = address
      @host = host
      @port = port
      @socket = socket
    end

    def announcement
      Messages::Peer.new(
        address: address.to_s,
        host: host,
        port: port)
    end

    def send(message)
      @socket.send(message, 0)
    end

    def recv(max_bytes)
      @socket.recv(max_bytes, 0)
    end

    def disconnect
      @socket.close
    end

    def inspect
      "<Comm::Peer address=#{address}>"
    end
  end
end
