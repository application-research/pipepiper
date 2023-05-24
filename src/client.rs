use super::args::Args;
use quinn::{ClientConfig, Connection, Endpoint};
use simple_eyre::{eyre::WrapErr, Result};
use std::{io::Write, sync::Arc};
use tokio::io::{stdin, AsyncReadExt, BufReader};

const STDIN_READER_BUFFER_SIZE: usize = 128 * (1 << 20);

pub async fn start_client(args: Args) -> Result<()> {
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(configure_client());
    let mut connection = endpoint.connect(args.addr, "remote")?.await?;

    send_stream(&mut connection).await?;
    // Dropping handles allows the corresponding objects to automatically shut down
    drop(connection);
    // Make sure the server has a chance to clean up
    endpoint.wait_idle().await;

    Ok(())
}

async fn send_stream(connection: &mut Connection) -> Result<()> {
    let mut send = connection.open_uni().await?;
    let mut buffer = Vec::with_capacity(STDIN_READER_BUFFER_SIZE);
    let mut reader = BufReader::new(stdin());

    loop {
        let read = reader.read_buf(&mut buffer).await?;
        if read == 0 {
            break;
        }

        send.write_all(&buffer[..read]).await?;
        buffer.clear();
    }

    send.finish().await?;

    Ok(())
}

// Dummy certificate verifier that treats any certificate as valid.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

fn configure_client() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}
