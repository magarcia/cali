class Cali < Formula
  desc "Calendar CLI - View your calendar events in the terminal"
  homepage "https://github.com/magarcia/cali"
  version "PLACEHOLDER_VERSION"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_ARM64_SHA256"
    else
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_INTEL_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER_LINUX_X86_SHA256"
    end
  end

  def install
    bin.install "cali"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cali --version")
  end
end
