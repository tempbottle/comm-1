module Comm
  class MessageRegistry
    def initialize
      @messages = Set.new
    end

    def add(message)
      if @messages.add?(message)
        yield message if block_given?
      end
    end
  end
end
