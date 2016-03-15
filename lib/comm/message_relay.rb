require 'timers'

module Comm
  class MessageRelay
    include Celluloid::IO
    include Celluloid::Internals::Logger

    class Scheduler
      attr_reader :count, :relay_at

      def initialize
        @count = 0
        @relay_at = Time.now.to_i
      end

      def increment
        @count += 1
        @relay_at = Time.now.to_i + delay
      end

      private

      def delay
        @count ** 2 - 1
      end
    end

    def initialize(node)
      @node = node
      @messages = Hash.new
    end

    def add(message)
      @messages.fetch(message) do
        puts "-> Adding message to relay"
        @messages[message] = Scheduler.new
      end
    end

    def run
      message, scheduler = @messages.select do |_, s|
        s.relay_at <= Time.now.to_i
      end.max_by { |_, s| s.relay_at }

      if message && scheduler
        info "-> Relay #{message.inspect}, #{scheduler.inspect}"
        @node.broadcast(message)
        scheduler.increment
      end

      async.run
    end
  end
end
