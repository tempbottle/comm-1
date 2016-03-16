require 'timers'

module Comm
  class MessageRelay
    include Celluloid::IO
    include Celluloid::Internals::Logger

    def initialize(node)
      @node = node
      @messages = Hash.new(0)
      @timer_group = Timers::Group.new
      @timers = {}
    end

    def add(message)
      return if relaying?(message)

      info "-> Adding message to relay"
      async.relay_message(message)
    end

    def relay_message(message)
      @timers[message] = @timer_group.after(delay_for(message)) do
        info "-> Relaying message"
        @timers.delete(message)
        @messages[message] += 1
        @node.broadcast(message)
        async.add(message)
      end
    end

    def run
      @timer_group.wait
      async.run
    end

    private

    def delay_for(message)
      @messages[message] ** 2 - 1
    end

    def relaying?(message)
      @timers.has_key?(message)
    end
  end
end
