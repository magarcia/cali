class Cali < Formula
  desc "Calendar CLI - View your calendar events in the terminal"
  homepage "https://github.com/magarcia/cali"
  version "0.3.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "d419bcc378ce34a750119a2fca48ac76ad150f4225d051eb25ede67bccc80651"
    else
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "4e588ed1f936cab14881dcacfc629d224041e51b21265e09c549ffbc3b28ac5f"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "7c25efb839dd415749d502d01fa677549f090a6d1e62696690596695577afa53"
    end
  end

  def install
    bin.install "cali"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cali --version")
  end
end
