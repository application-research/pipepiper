use super::args::Args;
use bytes::BufMut;
use quinn::{Connection, Endpoint, ServerConfig};
use simple_eyre::{
    eyre::{eyre, WrapErr},
    Result,
};
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufWriter};

const RECV_CHUNK_SIZE: usize = 128 * (1 << 20);

pub async fn start_server(args: Args) -> Result<()> {
    let (server_config, _server_cert) = configure_server()?;
    let mut endpoint = Endpoint::server(server_config, args.addr)?;

    let mut connection = endpoint
        .accept()
        .await
        .ok_or(eyre!("end point is closed"))?
        .await?;

    receive_stream(&mut connection).await?;
    Ok(())
}

async fn receive_stream(connection: &mut Connection) -> Result<()> {
    let mut recv = connection.accept_uni().await?;
    let mut writer = BufWriter::new(tokio::io::stdout());

    while let Some(mut chunk) = recv.read_chunk(RECV_CHUNK_SIZE, true).await? {
        writer.write_all_buf(&mut chunk.bytes).await?;
    }

    writer.flush().await?;
    Ok(())
}

/// Returns default server configuration along with its certificate.
fn configure_server() -> Result<(ServerConfig, Vec<u8>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();

    Ok((server_config, cert_der))
}
