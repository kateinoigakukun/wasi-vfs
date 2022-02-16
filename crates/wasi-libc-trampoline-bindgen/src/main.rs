use structopt::StructOpt;
use wasi_libc_trampoline_bindgen::App;

fn main() {
    App::from_args().execute().unwrap();
}
