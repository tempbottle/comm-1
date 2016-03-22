require 'logger'

module Comm
  class NullClient
    def logger
      @logger ||= Logger.new(STDOUT)
    end

    def add_message(message)
    end

    def update_peers(peers)
    end
  end
end
