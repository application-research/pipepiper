#![allow(warnings)]

mod args;

#[tokio::main]
async fn main() {
    let args = args::parse_args();

    match args.mode {
        args::PiperMode::Server => todo!(),
        args::PiperMode::Client => todo!(),
    };
}
