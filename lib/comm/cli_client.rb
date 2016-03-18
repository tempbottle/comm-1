require 'curses'
require 'set'

module Comm
  class CliClient
    def initialize(node)
      @node = node
      @node.attach_client(self)

      @window = Curses::Window.new(0, 0, 0, 0)
      @transcript = @window.subwin(Curses.lines - 1, Curses.cols - 40, 0, 0)
      @transcript.scrollok(true)
      @transcript.setpos(@transcript.maxy - 1, 0)
      @input = @window.subwin(1, 0, Curses.lines - 1, 0)
      @contacts = @window.subwin(Curses.lines - 1, 40, 0, Curses.cols - 40)
      @contacts.addstr("Hello")
      @contacts.refresh
      @peers = Set.new
    end

    def add_message(message)
      @transcript.addstr("#{message.text}")
      @transcript.scroll
      @transcript.setpos(@transcript.cury, 0)
      @transcript.refresh
    end

    def add_peer(peer)
      @peers |= [peer]
      render_peers
    end

    def remove_peer(peer)
      @peers.delete(peer)
      render_peers
    end

    def run
      loop do
        text = @input.getstr
        @input.clear
        recipient = 'fe05bcdcdc4928012781a5f1a2a77cbb5398e106'
        message = Comm::Messages::Chat.new(
          address: Comm::Address.for_content(recipient + text).to_s,
          recipient: recipient,
          text: text)
        node.broadcast(message)
      end
    end

    private

    attr_reader :node

    def render_peers
      @contacts.clear
      @peers.each do |peer|
        @contacts.addstr(peer.address.to_s)
      end
      @contacts.refresh
    end
  end
end
