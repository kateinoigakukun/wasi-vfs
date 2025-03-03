const { WASI } = require("wasi");
const fs = require("fs");
const process = require("process");
const path = require("path");

const args = process.argv.slice(2);

const preopens = {};
while (args.length > 0 && args[0].startsWith("--dir=")) {
  // Parse --dir=/foo::/bar
  const dir = args.shift().substring("--dir=".length);
  const [host, guest] = dir.split("::");
  preopens[guest] = host;
}

const buffer = fs.readFileSync(args[0]);
const stage = Number(args[1]);

const wasi = new WASI({
  version: "preview1",
  env: { ...process.env },
  preopens
});
const m = new WebAssembly.Module(buffer);
const i = new WebAssembly.Instance(m, {
  wasi_snapshot_preview1: wasi.wasiImport,
});

wasi.initialize(i);
i.exports.check(stage);
