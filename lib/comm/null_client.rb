require 'logger'

module Comm
  class NullClient
    def logger
      @logger ||= Logger.new(STDOUT)
    end

    def add_message(message)
    end

    def add_peer(peer)
    end
  end
end
