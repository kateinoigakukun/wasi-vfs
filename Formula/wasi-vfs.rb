class WasiVfs < Formula
  desc "A virtual filesystem layer for WASI."
  homepage "https://github.com/kateinoigakukun/wasi-vfs"
  url "https://github.com/kateinoigakukun/wasi-vfs.git", tag: "v0.1.1", using: :git
  head "https://github.com/kateinoigakukun/wasi-vfs.git", branch: "main"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install",
           "--bin", "wasi-vfs",
           "--path", "./crates/wasi-vfs-cli",
           "--root", prefix
  end

  test do
    system bin/"wasi-vfs", "--version"
  end
end
