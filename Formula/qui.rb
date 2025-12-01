class Qui < Formula
  desc "Kubernetes Terminal UI - An intuitive TUI for managing Kubernetes clusters"
  homepage "https://github.com/taufiksoleh/qui"
  version "0.0.4"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.4/qui-macos-x86_64.tar.gz"
      sha256 "689501ed0749deaf6c8d09c7df51fd70dd82e8c7810eb3989d327c79943c34ea" # Will be updated by release workflow
    elsif Hardware::CPU.arm?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.4/qui-macos-aarch64.tar.gz"
      sha256 "5fab5853cef9038d1e9beae972d8257f8d48704b5cda3e4c386d3b2ade0ad07f" # Will be updated by release workflow
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/taufiksoleh/qui/releases/download/v0.0.4/qui-linux-x86_64.tar.gz"
      sha256 "d3b8746dd86ad9ba0fc73b63afc84bc8dc7c0716df260d2ba0b2180a4c058183" # Will be updated by release workflow
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
