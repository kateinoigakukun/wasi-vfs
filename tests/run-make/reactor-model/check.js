const { WASI } = require("wasi");
const fs = require("fs");
const process = require("process");

const buffer = fs.readFileSync(process.argv[2]);

const wasi = new WASI({
  env: { ...process.env },
});
const m = new WebAssembly.Module(buffer);
const i = new WebAssembly.Instance(m, {
  wasi_snapshot_preview1: wasi.wasiImport,
});

wasi.initialize(i);
i.exports.check();
