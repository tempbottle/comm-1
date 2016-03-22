module Comm
  class Peer
    attr_reader :address, :host, :port

    def initialize(address:, host:, port:)
      @address = address
      @host = host
      @port = port
    end

    def ==(other)
      self.address == other.address
    end

    def <=>(other)
      address <=> other.address
    end

    def hash
      [self.class, address].hash
    end

    def inspect
      "<Comm::Peer address=#{address} host=#{host} port=#{port}>"
    end

    def send(message)
      TCPSocket.open(host, port) do |socket|
        socket.send(message, 0)
      end
    rescue Errno::ECONNREFUSED, Errno::ETIMEDOUT
    end

    def serialize
      Messages::Peer.new(
        address: address.to_s,
        host: host,
        port: port)
    end

    protected

    def eql?(other)
      self == other
    end
  end
end
