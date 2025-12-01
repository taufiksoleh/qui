class Qui < Formula
  desc "Kubernetes Terminal UI - An intuitive TUI for managing Kubernetes clusters"
  homepage "https://github.com/taufiksoleh/qui"
  version "0.0.3"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.3/qui-macos-x86_64.tar.gz"
      sha256 "a96c561be5cac008f20d475ad1a654d23dbaee755a364ccbf243c7e2cc2de82a" # Will be updated by release workflow
    elsif Hardware::CPU.arm?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.3/qui-macos-aarch64.tar.gz"
      sha256 "6ca68c13785059837ca9c939fe14f5758f7b1eff8c1e9c40eb3b0b69c501cb21" # Will be updated by release workflow
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.3/qui-linux-x86_64.tar.gz"
      sha256 "5c3738a4322736a69403ba74104e4c1757dae44ac675d1df76b8c7ecdb98b628" # Will be updated by release workflow
    end
  end

  def install
    bin.install "qui"
  end

  test do
    # Test that the binary exists and is executable
    assert_predicate bin/"qui", :exist?
    assert_predicate bin/"qui", :executable?
  end
end
