class Peep < Formula
  desc "Peep into your AI coding agents — terminal dashboard with pixel-art characters"
  homepage "https://github.com/jsleemaster/peep"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/jsleemaster/peep/releases/download/v#{version}/peep-macos-arm64.tar.gz"
      # sha256 will be filled after first release
    else
      url "https://github.com/jsleemaster/peep/releases/download/v#{version}/peep-macos-intel.tar.gz"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/jsleemaster/peep/releases/download/v#{version}/peep-linux-arm64.tar.gz"
    else
      url "https://github.com/jsleemaster/peep/releases/download/v#{version}/peep-linux-x86_64.tar.gz"
    end
  end

  def install
    bin.install "peep"
  end

  test do
    assert_match "peep", shell_output("#{bin}/peep --help")
  end
end
