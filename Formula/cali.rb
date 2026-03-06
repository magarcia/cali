class Cali < Formula
  desc "Calendar CLI - View your calendar events in the terminal"
  homepage "https://github.com/magarcia/cali"
  version "0.3.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "ad4e3bd8215106e7ef5fedf245bfe35ccd1b0b32c1bfcb67c07462a9a23c1866"
    else
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "fadcdf4574d826da6162dcdddbc9a88bdb3535e1131c9988c231710bde4201ee"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "892866da7a03825455d96f009853618150b3dfb06c13c4bce12a416ad4b41f25"
    end
  end

  def install
    bin.install "cali"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cali --version")
  end
end
