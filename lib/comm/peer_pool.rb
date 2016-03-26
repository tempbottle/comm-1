require 'set'
require 'forwardable'

module Comm
  class PeerPool
    extend Forwardable

    ADDRESS_LENGTH = 40

    def initialize(node, size: 10)
      @node = node
      @size = size
      @self_peer = node.self_peer
      @buckets = ADDRESS_LENGTH.times.map { |n| NodeList.new(@node, n) }.reverse
    end

    def add(peer, &on_add)
      bucket = @buckets.detect do |b|
        b.belongs?(peer)
      end and bucket.store(peer, &on_add)
    end

    def each(&block)
      @buckets.each { |bucket| bucket.each(&block) }
    end

    def empty?
      @buckets.all?(&:empty?)
    end

    def except(*exclusions)
      to_a - exclusions
    end

    def except_self
      except(@self_peer)
    end

    def nearest_to(address)
      sort_by { |p| p.address.distance_from(address) }
    end

    def remove(peer)
      #@peers.delete(peer)
    end

    def sample(n = nil)
      if n
        to_a.sample(n)
      else
        to_a.sample
      end
    end

    def serialize
      Messages::Peers.new(peers: to_a.map(&:serialize))
    end

    def to_a
      @buckets.flat_map(&:to_a)
    end

    private

    attr_reader :peers

    class NodeList
      extend Forwardable
      def_delegators :peers, :empty?, :to_a

      def initialize(node, n)
        @node = node
        @n = n
        @peers = Set.new
        @first_n = (n - 1).times.reduce(0) { |x, _| x << 1 | 1 } << ADDRESS_LENGTH - n + 1
        @nth = 1 << ADDRESS_LENGTH - n
      end

      def belongs?(candidate)
        (candidate.address.to_i & @first_n) == (@node.address.to_i & @first_n) &&
          (candidate.address.to_i & @nth) != (@node.address.to_i & @nth)
      end

      def each(&block)
        @peers.each(&block)
      end

      def store(candidate)
        if belongs?(candidate) && peers.add?(candidate) && peers.size < @node.degree
          puts "-> Added peer #{candidate.inspect} to bucket #{n}"
          yield candidate
        end
      end

      private

      attr_reader :n, :peers
    end
  end
end
