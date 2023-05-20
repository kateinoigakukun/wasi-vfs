class WasiVfs < Formula
  desc "A virtual filesystem layer for WASI."
  homepage "https://github.com/kateinoigakukun/wasi-vfs"
  url "https://github.com/kateinoigakukun/wasi-vfs.git", tag: "v0.2.0", using: :git
  head "https://github.com/kateinoigakukun/wasi-vfs.git", branch: "main"
  license "Apache-2.0" => { with: "LLVM-exception" }

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
