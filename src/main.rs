mod args;
mod client;
mod server;
mod utils;

#[tokio::main]
async fn main() -> simple_eyre::Result<()> {
    simple_eyre::install()?;

    let args = args::parse_args();
    let config = utils::program_config(args.config.as_ref())?;

    match args.mode {
        args::PiperMode::Client => client::start_client(args, config).await,
        args::PiperMode::Server => server::start_server(args, config).await,
    }
}
