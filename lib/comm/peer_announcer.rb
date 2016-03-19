require 'timers'

module Comm
  class PeerAnnouncer
    include Celluloid::IO
    include Celluloid::Internals::Logger

    ANNOUNCE_FREQUENCY = 10 # seconds

    def initialize(peers)
      @peers = peers
      @timers = Timers::Group.new
    end

    def run
      @timer = @timers.now_and_every(ANNOUNCE_FREQUENCY) do
        announce_a_peer
      end
      info 'Announcing peers'
      loop { @timers.wait }
    end

    def stop
      info 'Stopping peer announcer'
      @timer.cancel
    end

    private

    def announce_a_peer
      return if @peers.except_self.empty?

      peer = @peers.except_self.sample
      info "Announcing peers to #{peer.inspect}"
      peer.send(Messages.encode(@peers.serialize))
    end
  end
end
