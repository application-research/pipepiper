use config::Config;
use quinn::{congestion, ClientConfig, ServerConfig, TransportConfig, VarInt};
use simple_eyre::Result;
use std::{sync::Arc, time::Duration};

pub const DEFAULT_BUFFER_CAP: usize = 256 * (1 << 20);

pub fn client_config(config: &Config) -> Result<ClientConfig> {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let mut client_config = ClientConfig::new(Arc::new(crypto));
    let mut transport_config = quinn::TransportConfig::default();
    setup_transport_config(&mut transport_config, config)?;
    client_config.transport_config(Arc::new(transport_config));

    Ok(client_config)
}

pub fn server_config(config: &Config) -> Result<(ServerConfig, Vec<u8>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    setup_transport_config(transport_config, config)?;

    Ok((server_config, cert_der))
}

fn setup_transport_config(transport_config: &mut TransportConfig, config: &Config) -> Result<()> {
    // changing the congestion algorithm from within the config file is not yet supported
    transport_config.congestion_controller_factory(Arc::new(congestion::BbrConfig::default()));

    for (key, value) in config.clone().cache.into_table()? {
        match key.as_str() {
            "stream_receive_window" => {
                transport_config.stream_receive_window(VarInt::from_u64(value.into_uint()?)?);
            }
            "receive_window" => {
                transport_config.receive_window(VarInt::from_u64(value.into_uint()?)?);
            }
            "send_window" => {
                transport_config.send_window(value.into_uint()?);
            }
            "datagram_receive_buffer_size" => {
                transport_config.datagram_receive_buffer_size(Some(value.into_uint()? as usize));
            }
            "datagram_send_buffer_size" => {
                transport_config.datagram_send_buffer_size(value.into_uint()? as usize);
            }
            "max_idle_timeout" => {
                transport_config
                    .max_idle_timeout(Some(VarInt::from_u64(value.into_uint()? * 1000)?.into()));
            }
            "keep_alive_interval" => {
                transport_config.keep_alive_interval(Some(Duration::from_secs(value.into_uint()?)));
            }

            _ => (),
        }
    }

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

pub fn program_config(file_path: Option<&String>) -> Result<Config> {
    let mut builder = Config::builder();

    if let Some(file) = file_path {
        builder = builder.add_source(config::File::with_name(file));
    }
    builder = builder.set_default("buffer_cap", DEFAULT_BUFFER_CAP as u64)?;

    Ok(builder.build()?)
}
