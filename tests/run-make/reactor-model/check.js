const { WASI } = require("wasi");
const fs = require("fs");
const process = require("process");
const path = require("path");

const buffer = fs.readFileSync(process.argv[2]);

const wasi = new WASI({
  version: "preview1",
  env: { ...process.env },
  preopens: {
    "/run": path.join(__dirname, "mnt", "run"),
  }
});
const m = new WebAssembly.Module(buffer);
const i = new WebAssembly.Instance(m, {
  wasi_snapshot_preview1: wasi.wasiImport,
});

wasi.initialize(i);
i.exports.check();
