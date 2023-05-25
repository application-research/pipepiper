use super::args::Args;
use quinn::{Connection, Endpoint, ServerConfig};
use simple_eyre::{
    eyre::{eyre, WrapErr},
    Result,
};
use tokio::io::{AsyncWriteExt, BufWriter};

const RECV_CHUNK_SIZE: usize = 128 * (1 << 20);

pub async fn start_server(args: Args) -> Result<()> {
    let (server_config, _server_cert) =
        configure_server().wrap_err("failed to create server config")?;
    let endpoint =
        Endpoint::server(server_config, args.addr).wrap_err("failed to create server endpoint")?;

    let mut connection = endpoint
        .accept()
        .await
        .ok_or(eyre!("end point is closed"))?
        .await
        .wrap_err("failed bind the the server socket")?;

    receive_stream(&mut connection).await?;
    Ok(())
}

async fn receive_stream(connection: &mut Connection) -> Result<()> {
    let mut recv = connection
        .accept_uni()
        .await
        .wrap_err("failed to accept a uni-directional stream")?;
    let mut writer = BufWriter::new(tokio::io::stdout());

    while let Some(mut chunk) = recv
        .read_chunk(RECV_CHUNK_SIZE, true)
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

fn configure_server() -> Result<(ServerConfig, Vec<u8>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;

    Ok((server_config, cert_der))
}
