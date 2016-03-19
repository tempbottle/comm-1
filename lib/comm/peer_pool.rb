require 'set'
require 'forwardable'

module Comm
  class PeerPool
    extend Forwardable
    def_delegators :peers, :each, :empty?, :map

    def initialize(node, peers: Set.new([node.self_peer]), size: 2)
      @node = node
      @peers = peers
      @size = size
      @self_peer = node.self_peer
    end

    def add(peer, &on_add)
      if @peers.add?(peer)
        if @peers.include?(peer)
          info "-> Added peer #{peer.inspect}"
          yield peer
        end
      end
    end

    def except(*exclusions)
      peers = @peers - exclusions
      self.class.new(@node, peers: peers, size: @size)
    end

    def except_self
      except(@self_peer)
    end

    def nearest_to(address)
      @peers.sort_by { |p| p.address.distance_from(address) }
    end

    def remove(peer)
      @peers.delete(peer)
    end

    def sample(*n)
      @peers.to_a.sample(*n)
    end

    def serialize
      Messages::Peers(peers: @peers.map(&:serialize))
    end

    private

    attr_reader :peers

    def cull
      @peers = nearest_to(@node.address).first(@size).to_set
    end
  end
end
