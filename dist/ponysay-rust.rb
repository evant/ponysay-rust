class PonysayRust < Formula
  desc "Bare-bones implementation of ponysay in rust"
  homepage "https://github.com/evant/ponysay-rust"
  url "file:///Users/evantatarka/rust/ponysay", :using => :git
  version "0.1"
  sha256 ""

  depends_on "rust" => :build

  conflicts_with "ponysay", :because => "it ships the ponysay binary"

  def install
    system "make", "install", "PREFIX=#{prefix}"
  end

  test do
    system "make", "check"
  end
end
