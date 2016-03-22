require 'set'
require 'forwardable'

module Comm
  class PeerPool
    extend Forwardable
    def_delegators :peers, :each, :empty?, :map, :to_a

    def initialize(node, peers: Set.new([node.self_peer]), size: 2)
      @node = node
      @peers = peers
      @size = size
      @self_peer = node.self_peer
    end

    def add(peer, &on_add)
      if @peers.add?(peer)
        if @peers.include?(peer)
          puts "-> Added peer #{peer.inspect}"
          yield peer
        end
      end
    end

    def except(*exclusions)
      subset(@peers - exclusions)
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

    def sample(n = nil)
      if n
        subset(@peers.to_a.sample(n))
      else
        @peers.to_a.sample
      end
    end

    def serialize
      Messages::Peers.new(peers: @peers.map(&:serialize))
    end

    private

    attr_reader :peers

    def cull
      @peers = nearest_to(@node.address).first(@size).to_set
    end

    def subset(peers)
      self.class.new(@node, peers: peers, size: @size)
    end
  end
end
