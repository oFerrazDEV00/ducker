//! Cliente HTTP(S) do protocolo LocalSend v2 (lado remetente).

use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::header::CONTENT_LENGTH;
use tokio_util::io::ReaderStream;

use crate::error::{DuckerError, Result};
use crate::model::{DeviceInfo, PrepareUploadRequest, PrepareUploadResponse, Protocol, API_PREFIX};
use crate::tls;

const CHUNK: usize = 64 * 1024;

pub type ProgressFn = Arc<dyn Fn(u64) + Send + Sync>;

pub struct PeerClient {
    http: reqwest::Client,
    base: String,
}

impl PeerClient {
    /// `expected_fingerprint`: se `Some`, o certificado do servidor precisa ter esse SHA-256.
    /// `timeout`: tempo total por requisição (None = sem limite, usado em uploads).
    pub fn new(
        protocol: Protocol,
        ip: IpAddr,
        port: u16,
        expected_fingerprint: Option<String>,
        timeout: Option<Duration>,
    ) -> Result<Self> {
        tls::install_crypto_provider();
        let mut builder = reqwest::Client::builder()
            .use_preconfigured_tls(tls::client_config(expected_fingerprint))
            .connect_timeout(Duration::from_secs(5))
            .no_proxy();
        if let Some(t) = timeout {
            builder = builder.timeout(t);
        }
        let http = builder.build()?;
        let base = format!("{}://{}{}", protocol.scheme(), SocketAddr::new(ip, port), API_PREFIX);
        Ok(Self { http, base })
    }

    fn conn_err(e: reqwest::Error) -> DuckerError {
        DuckerError::ConnectionFailed(e.to_string())
    }

    pub async fn register(&self, me: &DeviceInfo) -> Result<DeviceInfo> {
        let resp = self
            .http
            .post(format!("{}/register", self.base))
            .json(me)
            .send()
            .await
            .map_err(Self::conn_err)?;
        if !resp.status().is_success() {
            return Err(DuckerError::ConnectionFailed(format!("register retornou HTTP {}", resp.status())));
        }
        Ok(resp.json().await?)
    }

    pub async fn info(&self) -> Result<DeviceInfo> {
        let resp = self.http.get(format!("{}/info", self.base)).send().await.map_err(Self::conn_err)?;
        if !resp.status().is_success() {
            return Err(DuckerError::ConnectionFailed(format!("info retornou HTTP {}", resp.status())));
        }
        Ok(resp.json().await?)
    }

    /// Retorna `Ok(None)` quando o receptor responde 204 (nada a transferir, ex: mensagem de texto).
    pub async fn prepare_upload(
        &self,
        req: &PrepareUploadRequest,
        pin: Option<&str>,
        expected_quac: Option<u32>,
    ) -> Result<Option<PrepareUploadResponse>> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(pin) = pin {
            query.push(("pin", pin.to_string()));
        }
        if let Some(q) = expected_quac {
            query.push(("quac", q.to_string()));
        }
        let resp = self
            .http
            .post(format!("{}/prepare-upload", self.base))
            .query(&query)
            .json(req)
            .send()
            .await
            .map_err(Self::conn_err)?;
        match resp.status().as_u16() {
            200 => Ok(Some(resp.json().await?)),
            204 => Ok(None),
            401 => Err(DuckerError::PinRequired),
            403 => {
                let body = resp.text().await.unwrap_or_default();
                if body.contains("DESTINATION_ID_MISMATCH") {
                    Err(DuckerError::DestinationIdMismatch)
                } else {
                    Err(DuckerError::Rejected)
                }
            }
            409 => Err(DuckerError::Busy),
            s => Err(DuckerError::TransferFailed(format!("prepare-upload retornou HTTP {s}"))),
        }
    }

    async fn finish_upload(resp: reqwest::Response) -> Result<()> {
        match resp.status().as_u16() {
            200 | 204 => Ok(()),
            403 => Err(DuckerError::TransferFailed("token ou IP inválido".into())),
            409 => Err(DuckerError::Busy),
            422 => Err(DuckerError::ChecksumMismatch),
            s => Err(DuckerError::TransferFailed(format!("upload retornou HTTP {s}"))),
        }
    }

    pub async fn upload_file(
        &self,
        session_id: &str,
        file_id: &str,
        token: &str,
        path: &Path,
        size: u64,
        progress: ProgressFn,
    ) -> Result<()> {
        let file = tokio::fs::File::open(path).await?;
        let sent = Arc::new(AtomicU64::new(0));
        let stream = ReaderStream::with_capacity(file, CHUNK).map(move |chunk| {
            if let Ok(bytes) = &chunk {
                let total = sent.fetch_add(bytes.len() as u64, Ordering::SeqCst) + bytes.len() as u64;
                progress(total);
            }
            chunk
        });
        let resp = self
            .http
            .post(format!("{}/upload", self.base))
            .query(&[("sessionId", session_id), ("fileId", file_id), ("token", token)])
            .header(CONTENT_LENGTH, size)
            .body(reqwest::Body::wrap_stream(stream))
            .send()
            .await
            .map_err(Self::conn_err)?;
        Self::finish_upload(resp).await
    }

    pub async fn upload_bytes(&self, session_id: &str, file_id: &str, token: &str, data: Vec<u8>) -> Result<()> {
        let resp = self
            .http
            .post(format!("{}/upload", self.base))
            .query(&[("sessionId", session_id), ("fileId", file_id), ("token", token)])
            .body(data)
            .send()
            .await
            .map_err(Self::conn_err)?;
        Self::finish_upload(resp).await
    }

    pub async fn cancel(&self, session_id: &str) -> Result<()> {
        self.http
            .post(format!("{}/cancel", self.base))
            .query(&[("sessionId", session_id)])
            .send()
            .await
            .map_err(Self::conn_err)?;
        Ok(())
    }
}
