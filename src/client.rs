use super::args::Args;
use crate::utils::client_config;
use config::Config;
use quinn::{Connection, Endpoint};
use simple_eyre::{eyre::WrapErr, Result};
use tokio::io::{AsyncReadExt, BufReader};

pub async fn start_client(args: Args, config: Config) -> Result<()> {
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(
        client_config(&config).wrap_err("failed to create client configuration")?,
    );
    let mut connection = endpoint
        .connect(args.addr, "remote")
        .wrap_err("possibly malformed configuration")?
        .await
        .wrap_err("failed to connect to server")?;

    send_stream(
        &mut connection,
        config
            .get_int("default_cap")
            .map(|val| val as usize)
            .unwrap_or(crate::utils::DEFAULT_BUFFER_CAP),
    )
    .await?;

    // Dropping handles allows the corresponding objects to automatically shut down
    drop(connection);
    // Make sure the server has a chance to clean up
    endpoint.wait_idle().await;

    Ok(())
}

async fn send_stream(connection: &mut Connection, buf_size: usize) -> Result<()> {
    let mut send = connection
        .open_uni()
        .await
        .wrap_err("failed to open a uni-directional stream to server")?;
    let mut buffer = Vec::with_capacity(buf_size);
    let mut reader = BufReader::new(tokio::io::stdin());

    loop {
        let read = reader
            .read_buf(&mut buffer)
            .await
            .wrap_err("failed to read from stdin")?;
        if read == 0 {
            break;
        }

        send.write_all(&buffer[..read])
            .await
            .wrap_err("failed to send data to server")?;
        buffer.clear();
    }

    send.finish()
        .await
        .wrap_err("failed to shutdown the connection gracefully")
}
