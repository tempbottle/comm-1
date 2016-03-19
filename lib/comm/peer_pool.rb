require 'set'
module Comm
  class PeerPool
    def initialize(size: 2)
      @peers = Set.new
      @size = size
    end

    def add(peer, &on_add)
      if @peers.add?(peer)
        if @peers.include?(peer)
          puts "-> Added peer #{peer.inspect}"
          yield peer
        end
      end
    end

    def each(&block)
      return enum_for(__method__) unless block_given?

      @peers.each(&block)
    end

    def remove(peer)
      @peers.delete(peer)
    end
  end
end
