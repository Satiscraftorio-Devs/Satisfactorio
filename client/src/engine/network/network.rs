use quinn::{RecvStream, SendStream};
use shared::network::crypto::compute_shared_secret;
use shared::network::messages::Paquet;
use shared::network::network_protocol::{create_codec, EncryptedCodec};

pub struct ClientConnection {
    codec: EncryptedCodec,
}

impl ClientConnection {
    pub fn new(server_id: [u8; 16]) -> Self {
        let codec = create_codec(compute_shared_secret(&server_id, b"client"));
        Self { codec }
    }

    pub async fn send_packet(&mut self, send: &mut SendStream, packet: Paquet) -> Result<(), String> {
        let data = self.codec.encode(&packet);
        send.write_all(&data).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn receive_packet(&mut self, recv: &mut RecvStream) -> Result<Paquet, String> {
        use tokio::io::AsyncReadExt;

        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await.map_err(|e| e.to_string())?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > 65536 {
            return Err(format!("Paquet trop grand: {}", len));
        }

        let mut data = vec![0u8; len];
        recv.read_exact(&mut data).await.map_err(|e| e.to_string())?;

        let packet = self.codec.decode(&data)?;
        Ok(packet)
    }
}

#[derive(Debug)]
struct DangerousSkipVerify;

impl rustls::client::danger::ServerCertVerifier for DangerousSkipVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
        ]
    }
}
