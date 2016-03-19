require 'set'

module Comm
  class PeerPool
    def initialize(node, size: 2)
      @node = node
      @peers = Set.new
      @size = size
    end

    def add(peer, &on_add)
      if @peers.add?(peer)
        cull
        if @peers.include?(peer)
          info "-> Added peer #{peer.inspect}"
          yield peer
        end
      end
    end

    def each(&block)
      return enum_for(__method__) unless block_given?

      @peers.each(&block)
    end

    def nearest_to(address)
      @peers.sort_by { |p| p.address.distance_from(address) }
    end

    def remove(peer)
      @peers.delete(peer)
    end

    private

    def cull
      @peers = nearest_to(@node.address).first(@size).to_set
    end
  end
end
