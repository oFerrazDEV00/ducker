//! TLS: certificado autoassinado, fingerprint SHA-256, config do servidor e
//! verificador do cliente (aceita certificados autoassinados, com pinning opcional).

use std::sync::{Arc, Once};

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use sha2::{Digest, Sha256};

use crate::error::{DuckerError, Result};

static INIT: Once = Once::new();

/// Instala o provider `ring` como padrão do processo. Idempotente.
pub fn install_crypto_provider() {
    INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

/// Gera um par (cert_pem, key_pem) autoassinado.
pub fn generate_self_signed() -> Result<(String, String)> {
    let ck = rcgen::generate_simple_self_signed(vec!["localsend".to_string(), "ducker".to_string()])
        .map_err(|e| DuckerError::Tls(e.to_string()))?;
    Ok((ck.cert.pem(), ck.key_pair.serialize_pem()))
}

fn parse_cert_der(cert_pem: &str) -> Result<CertificateDer<'static>> {
    let mut reader = std::io::BufReader::new(cert_pem.as_bytes());
    let cert_res = {
        let mut iter = rustls_pemfile::certs(&mut reader);
        iter.next()
            .ok_or_else(|| DuckerError::Tls("PEM sem certificado".into()))?
    };
    cert_res.map_err(|e| DuckerError::Tls(e.to_string()))
}

fn parse_key_der(key_pem: &str) -> Result<PrivateKeyDer<'static>> {
    let mut reader = std::io::BufReader::new(key_pem.as_bytes());
    let key_opt = rustls_pemfile::private_key(&mut reader)
        .map_err(|e| DuckerError::Tls(e.to_string()))?;
    key_opt.ok_or_else(|| DuckerError::Tls("PEM sem chave privada".into()))
}

/// SHA-256 (hex maiúsculo) dos bytes DER de um certificado.
pub fn fingerprint_der(der: &[u8]) -> String {
    hex::encode_upper(Sha256::digest(der))
}

/// Fingerprint de um certificado PEM (padrão LocalSend em modo HTTPS).
pub fn fingerprint_pem(cert_pem: &str) -> Result<String> {
    Ok(fingerprint_der(parse_cert_der(cert_pem)?.as_ref()))
}

/// Config rustls do servidor HTTPS.
pub fn server_config(cert_pem: &str, key_pem: &str) -> Result<Arc<rustls::ServerConfig>> {
    let cert = parse_cert_der(cert_pem)?;
    let key = parse_key_der(key_pem)?;
    let mut cfg = rustls::ServerConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .map_err(|e| DuckerError::Tls(e.to_string()))?
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .map_err(|e| DuckerError::Tls(e.to_string()))?;
    cfg.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(Arc::new(cfg))
}

/// Verificador que aceita certificados autoassinados. Se `expected_fingerprint`
/// for informado, o SHA-256 do certificado do servidor precisa coincidir (pinning).
#[derive(Debug)]
pub struct FingerprintVerifier {
    expected: Option<String>,
    provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for FingerprintVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        if let Some(expected) = &self.expected {
            let actual = fingerprint_der(end_entity.as_ref());
            if !actual.eq_ignore_ascii_case(expected) {
                return Err(rustls::Error::General(format!(
                    "fingerprint mismatch: esperado {expected}, recebido {actual}"
                )));
            }
        }
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.provider.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider.signature_verification_algorithms.supported_schemes()
    }
}

/// Config rustls do cliente com certificado de autenticação de cliente (mTLS exigido pelo LocalSend).
pub fn client_config(
    client_cert_key: Option<(&str, &str)>,
    expected_fingerprint: Option<String>,
) -> rustls::ClientConfig {
    let provider = provider();
    let verifier = Arc::new(FingerprintVerifier {
        // Fingerprints vazios (peers em HTTP ou sem fingerprint) não são verificados.
        expected: expected_fingerprint.filter(|f| f.len() == 64),
        provider: provider.clone(),
    });
    let builder = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("versões TLS padrão")
        .dangerous()
        .with_custom_certificate_verifier(verifier);

    let (cert_pem, key_pem) = match client_cert_key {
        Some((c, k)) => (c.to_string(), k.to_string()),
        None => generate_self_signed().expect("geração de cert do cliente"),
    };

    let cert = parse_cert_der(&cert_pem).expect("parse cert der");
    let key = parse_key_der(&key_pem).expect("parse key der");

    builder
        .with_client_auth_cert(vec![cert], key)
        .expect("configuração de mTLS do cliente")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cert_roundtrip_and_fingerprint() {
        install_crypto_provider();
        let (cert, key) = generate_self_signed().unwrap();
        let fp = fingerprint_pem(&cert).unwrap();
        assert_eq!(fp.len(), 64);
        assert!(server_config(&cert, &key).is_ok());
        let _ = client_config(Some((&cert, &key)), Some(fp));
    }
}
