use super::args::Args;
use quinn::{ClientConfig, Connection, Endpoint};
use simple_eyre::{eyre::WrapErr, Result};
use std::sync::Arc;
use tokio::io::{stdin, AsyncReadExt, BufReader};

const STDIN_READER_BUFFER_SIZE: usize = 128 * (1 << 20);

pub async fn start_client(args: Args) -> Result<()> {
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    let config = configure_client().wrap_err("failed to create client configuration")?;
    endpoint.set_default_client_config(config);
    let mut connection = endpoint
        .connect(args.addr, "remote")
        .wrap_err("possibly malformed configuration")?
        .await
        .wrap_err("failed to connect to server")?;

    send_stream(&mut connection).await?;

    // Dropping handles allows the corresponding objects to automatically shut down
    drop(connection);
    // Make sure the server has a chance to clean up
    endpoint.wait_idle().await;

    Ok(())
}

async fn send_stream(connection: &mut Connection) -> Result<()> {
    let mut send = connection
        .open_uni()
        .await
        .wrap_err("failed to open a uni-directional stream to server")?;
    let mut buffer = Vec::with_capacity(STDIN_READER_BUFFER_SIZE);
    let mut reader = BufReader::new(stdin());

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

fn configure_client() -> Result<ClientConfig> {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let mut config = ClientConfig::new(Arc::new(crypto));
    let transport_config = quinn::TransportConfig::default();
    config.transport_config(Arc::new(transport_config));

    Ok(config)
}
