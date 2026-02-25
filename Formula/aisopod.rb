class Aisopod < Formula
  desc "AI gateway and agent orchestration platform"
  homepage "https://github.com/AIsopod/aisopod"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/AIsopod/aisopod/releases/download/v#{version}/aisopod-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    else
      url "https://github.com/AIsopod/aisopod/releases/download/v#{version}/aisopod-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86"
    end
  end

  on_linux do
    url "https://github.com/AIsopod/aisopod/releases/download/v#{version}/aisopod-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX"
  end

  def install
    bin.install "aisopod"
  end

  test do
    assert_match "aisopod", shell_output("#{bin}/aisopod --version")
  end
end
