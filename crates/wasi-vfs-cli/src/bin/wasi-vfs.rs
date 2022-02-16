use structopt::StructOpt;
use wasi_vfs_cli::App;

fn main() {
    App::from_args().execute().unwrap();
}
