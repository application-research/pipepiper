mod args;
mod client;
mod server;

#[tokio::main]
async fn main() -> simple_eyre::Result<()> {
    simple_eyre::install()?;

    let args = args::parse_args();

    match args.mode {
        args::PiperMode::Client => client::start_client(args).await,
        args::PiperMode::Server => server::start_server(args).await,
    }
}
