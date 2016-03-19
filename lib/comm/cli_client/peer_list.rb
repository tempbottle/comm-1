module Comm
  class CliClient
    class PeerList
      def initialize(window)
        @height = (Curses.lines - 1) / 2
        @width = 42
        @top = 0
        @left = Curses.cols - 42
        @window = window.subwin(@height, @width, @top, @left)
        @peers = []
        @index = 0
        render
      end

      def add(peer)
        @peers |= [peer]
        @peers.sort!
        render
      end

      def remove(peer)
        @peers.delete(peer)
        render
      end

      def select_next
        @index = (@index + 1) % @peers.size
        render
      end

      def select_previous
        @index = @index - 1
        if @index < 0
          @index = @peers.size - 1
        end
        render
      end

      def selected
        @peers.fetch(@index)
      end

      private

      def render
        @window.clear
        @window.addstr("Peers\n")
        @window.addstr('-' * @width)
        @peers.each do |peer|
          if selected?(peer)
            @window.addstr("> #{peer.address.to_s}")
          else
            @window.addstr("  #{peer.address.to_s}")
          end
        end
        @window.refresh
      end

      def selected?(peer)
        peer == selected
      end
    end
  end
end
