use super::args::Args;
use crate::utils::server_config;
use config::Config;
use quinn::{Connection, Endpoint};
use simple_eyre::{
    eyre::{eyre, WrapErr},
    Result,
};
use tokio::io::{AsyncWriteExt, BufWriter};

pub async fn start_server(args: Args, config: Config) -> Result<()> {
    let (server_config, _server_cert) =
        server_config(&config).wrap_err("failed to create server config")?;
    let endpoint =
        Endpoint::server(server_config, args.addr).wrap_err("failed to create server endpoint")?;

    let mut connection = endpoint
        .accept()
        .await
        .ok_or(eyre!("end point is closed"))?
        .await
        .wrap_err("failed bind the the server socket")?;

    receive_stream(
        &mut connection,
        config
            .get_int("buffer_cap")
            .map(|val| val as usize)
            .unwrap_or(crate::utils::DEFAULT_BUFFER_CAP),
    )
    .await?;
    Ok(())
}

async fn receive_stream(connection: &mut Connection, buf_size: usize) -> Result<()> {
    let mut recv = connection
        .accept_uni()
        .await
        .wrap_err("failed to accept a uni-directional stream")?;
    let mut writer = BufWriter::new(tokio::io::stdout());

    while let Some(mut chunk) = recv
        .read_chunk(buf_size, true)
        .await
        .wrap_err("failed to read a chunk from uni-directional stream")?
    {
        writer
            .write_all_buf(&mut chunk.bytes)
            .await
            .wrap_err("failed to do a buffered write to stdout")?;
    }

    writer
        .flush()
        .await
        .wrap_err("failed to flush out to stdout")
}
