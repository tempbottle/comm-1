require 'timers'

module Comm
  class MessageRelay
    include Celluloid::IO
    include Celluloid::Internals::Logger

    THRESHOLD = 4
    RELAY_FREQUENCY = 10

    class Scheduler
      attr_reader :count, :scheduled

      def initialize
        @count = 0
        @scheduled = Time.now.to_i
      end

      def increment
        @count += 1
        @scheduled = Time.now.to_i + delay
      end

      private

      def delay
        @count ** 2 - 1
      end
    end

    def initialize(node)
      @node = node
      @messages = {}
      @timers = Timers::Group.new
    end

    def add(message)
      return if @messages.has_key?(message)

      info "-> Adding message to relay"
      @messages[message] = Scheduler.new
    end

    def relay_message(message)
      return if @messages[message].count > THRESHOLD

      info "-> Relaying message #{message.inspect}"
      @node.broadcast(message)
      @messages[message].increment
    end

    def run
      @timer = @timers.now_and_every(RELAY_FREQUENCY) do
        @messages.each do |m, s|
          if s.scheduled <= Time.now.to_i
            relay_message(m)
          end
        end
      end

      loop { @timers.wait; sleep 0 }
    end

    def stop
      info 'Stopping message relay'
      @timer.cancel
    end
  end
end
