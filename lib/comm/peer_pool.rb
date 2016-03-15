module Comm
  class PeerPool
    def initialize(size: 2)
      @peers = {}
      @size = size
    end

    def add(peer, &on_add)
      return if @peers.has_key?(peer.address)
      return if @peers.size >= @size

      puts "-> Adding peer #{peer.inspect}"
      @peers[peer.address] = peer
      yield peer
    end

    def each(&block)
      return enum_for(__method__) unless block_given?

      @peers.values.each(&block)
    end

    def remove(peer)
      @peers.delete(peer.address)
    end
  end
end
