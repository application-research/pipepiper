#![allow(warnings)]

mod args;
mod client;
mod server;

#[tokio::main]
async fn main() {
    let args = args::parse_args();

    let ret = match args.mode {
        args::PiperMode::Client => client::start_client(args).await,
        args::PiperMode::Server => server::start_server(args).await,
    };

    if let Err(why) = ret {
        eprintln!("{why:#}");
        std::process::exit(1);
    }
}
