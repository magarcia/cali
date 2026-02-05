class Cali < Formula
  desc "Calendar CLI - View your calendar events in the terminal"
  homepage "https://github.com/magarcia/cali"
  version "0.2.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "fef7f01028e99954993490aab613a981a77c00706da38375f752470cbe83de90"
    else
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "dbf8423f60507f5aa1228b1978bd017920e91cd97378006bec9dafebcf24876b"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/magarcia/cali/releases/download/v#{version}/cali-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 ""
    end
  end

  def install
    bin.install "cali"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/cali --version")
  end
end
