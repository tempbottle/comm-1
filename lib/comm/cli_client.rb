require 'curses'
require 'set'

module Comm
  class CliClient
    def initialize(node)
      @node = node
      @node.attach_client(self)

      Curses.noecho
      @window = Curses::Window.new(0, 0, 0, 0)
      @transcript = @window.subwin(Curses.lines - 1, Curses.cols - 40, 0, 0)
      @transcript.scrollok(true)
      @transcript.setpos(@transcript.maxy - 1, 0)
      @input = @window.subwin(1, 0, Curses.lines - 1, 0)
      @contacts = @window.subwin(Curses.lines - 1, 40, 0, Curses.cols - 40)
      @contacts.addstr("Hello")
      @contacts.refresh
      @peers = []
      @peer_idx = 0
    end

    def add_message(message_payload)
      @transcript.addstr("<#{message_payload.sender}> #{message_payload.text}")
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

    def stop
      @stopped = true
    end

    def run
      @stopped = false
      buffer = ''

      @input.keypad = true
      loop do
        chr = @input.getch
        case chr
        when Curses::Key::UP
          previous_peer
        when Curses::Key::DOWN
          next_peer
        when Curses::Key::BACKSPACE, 127
          buffer.chop!
        when Curses::Key::ENTER, 10
          @input.clear
          @input.refresh
          @input.setpos(0, 0)
          node.deliver_chat(buffer, to: selected_peer)
          buffer.clear
        when String
          buffer << chr
        end

        @input.clear
        @input.addstr(buffer)
        @input.refresh
        break if @stopped
      end
    end

    private

    attr_reader :node

    def previous_peer
      @peer_idx -= 1
      render_peers
    end

    def next_peer
      @peer_idx += 1
      render_peers
    end

    def selected_peer?(peer)
      selected_peer == peer
    end

    def selected_peer
      @peers.fetch(@peer_idx)
    end

    def render_peers
      @contacts.clear
      @peers.each do |peer|
        if selected_peer?(peer)
          @contacts.standout
          @contacts.addstr(peer.address.to_s)
          @contacts.standend
        else
          @contacts.addstr(peer.address.to_s)
        end
      end
      @contacts.refresh
    end
  end
end
