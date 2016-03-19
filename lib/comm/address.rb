module Comm
  class Address
    def self.for_content(content)
      new(Digest::SHA1.hexdigest(content))
    end

    def initialize(address)
      @address = address
    end

    def ==(other)
      case other
      when Address
        to_s == other.to_s
      when String
        to_s == other
      end
    end

    def distance_from(other)
      to_i ^ other.to_i
    end

    def eql?(other)
      self == other
    end

    def hash
      [self.class, @address].hash
    end

    def <=>(other)
      to_i <=> other.to_i
    end

    def to_i
      @address.to_i(16)
    end

    def to_s
      @address
    end
  end
end
