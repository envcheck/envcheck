# Homebrew formula for envcheck
# To install: brew install envcheck/tap/envcheck

class Envcheck < Formula
  desc "Fast, modern Rust CLI for linting .env files and DevSecOps integrations"
  homepage "https://github.com/envcheck/envcheck"
  version "1.1.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/envcheck/envcheck/releases/download/v#{version}/envcheck-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_INTEL"
    end

    on_arm do
      url "https://github.com/envcheck/envcheck/releases/download/v#{version}/envcheck-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM"
    end
  end

  on_linux do
    url "https://github.com/envcheck/envcheck/releases/download/v#{version}/envcheck-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX"
  end

  def install
    bin.install "envcheck"
  end

  test do
    system "#{bin}/envcheck", "--version"
  end
end
