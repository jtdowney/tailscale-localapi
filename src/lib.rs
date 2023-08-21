use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use base64::Engine;
use http::{
    header::{AUTHORIZATION, HOST},
    Request, Response, Uri,
};
use hyper::{body::Buf, Body};
use tokio::net::{TcpSocket, UnixStream};
pub use types::*;

/// Definitions of types used in the tailscale API
pub mod types;

/// Error type for this crate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("connection failed")]
    IoError(#[from] io::Error),
    #[error("request failed")]
    HyperError(#[from] hyper::Error),
    #[error("http error")]
    HttpError(#[from] hyper::http::Error),
    #[error("unprocessible entity")]
    UnprocessableEntity,
    #[error("unable to parse json")]
    ParsingError(#[from] serde_json::Error),
    #[error("unable to parse certificate or key")]
    UnknownCertificateOrKey,
}

/// Result type for this crate
pub type Result<T> = std::result::Result<T, Error>;

/// Abstract trait for the tailscale API client
#[async_trait]
pub trait LocalApiClient {
    async fn get(&self, uri: Uri) -> Result<Response<Body>>;
}

/// Client for the local tailscaled socket
#[derive(Clone)]
pub struct LocalApi<T: LocalApiClient> {
    /// Path to the tailscaled socket
    client: T,
}

impl LocalApi<UnixStreamClient> {
    /// Create a new client for the local tailscaled from the path to the
    /// socket.
    pub fn new_with_socket_path<P: AsRef<Path>>(socket_path: P) -> Self {
        let socket_path = socket_path.as_ref().to_path_buf();
        let client = UnixStreamClient { socket_path };
        Self { client }
    }
}

impl LocalApi<TcpWithPasswordClient> {
    /// Create a new client for the local tailscaled from the TCP port and
    /// password.
    pub fn new_with_port_and_password<S: Into<String>>(port: u16, password: S) -> Self {
        let password = password.into();
        let client = TcpWithPasswordClient { port, password };
        Self { client }
    }
}

impl<T: LocalApiClient> LocalApi<T> {
    /// Get the certificate and key for a domain. The domain should be one of
    /// the valid domains for the local node.
    pub async fn certificate_pair(&self, domain: &str) -> Result<(PrivateKey, Vec<Certificate>)> {
        let response = self
            .client
            .get(
                format!("/localapi/v0/cert/{domain}?type=pair")
                    .parse()
                    .unwrap(),
            )
            .await?;

        let body = hyper::body::aggregate(response.into_body()).await?;
        let items = rustls_pemfile::read_all(&mut body.reader())?;
        let (certificates, mut private_keys) = items
            .into_iter()
            .map(|item| match item {
                rustls_pemfile::Item::ECKey(data)
                | rustls_pemfile::Item::PKCS8Key(data)
                | rustls_pemfile::Item::RSAKey(data) => Ok((false, data)),
                rustls_pemfile::Item::X509Certificate(data) => Ok((true, data)),
                _ => Err(Error::UnknownCertificateOrKey),
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .partition::<Vec<(bool, Vec<u8>)>, _>(|&(cert, _)| cert);

        let certificates = certificates
            .into_iter()
            .map(|(_, data)| Certificate(data))
            .collect();
        let (_, private_key_data) = private_keys.pop().ok_or(Error::UnknownCertificateOrKey)?;
        let private_key = PrivateKey(private_key_data);

        Ok((private_key, certificates))
    }

    /// Get the status of the local node.
    pub async fn status(&self) -> Result<Status> {
        let response = self
            .client
            .get(Uri::from_static("/localapi/v0/status"))
            .await?;
        let body = hyper::body::aggregate(response.into_body()).await?;
        let status = serde_json::de::from_reader(body.reader())?;

        Ok(status)
    }

    /// Request whois information for an address in the tailnet.
    pub async fn whois(&self, address: SocketAddr) -> Result<Whois> {
        let response = self
            .client
            .get(
                format!("/localapi/v0/whois?addr={address}")
                    .parse()
                    .unwrap(),
            )
            .await?;
        let body = hyper::body::aggregate(response.into_body()).await?;
        let whois = serde_json::de::from_reader(body.reader())?;

        Ok(whois)
    }
}

/// Client that connects to the local tailscaled over a unix socket. This is
/// used on Linux and other Unix-like systems.
pub struct UnixStreamClient {
    socket_path: PathBuf,
}

#[async_trait]
impl LocalApiClient for UnixStreamClient {
    async fn get(&self, uri: Uri) -> Result<Response<Body>> {
        let request = Request::builder()
            .method("GET")
            .header(HOST, "local-tailscaled.sock")
            .uri(uri)
            .body(Body::empty())?;

        let response = self.request(request).await?;
        Ok(response)
    }
}

impl UnixStreamClient {
    async fn request(&self, request: Request<Body>) -> Result<Response<Body>> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let (mut request_sender, connection) = hyper::client::conn::handshake(stream).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Error in connection: {}", e);
            }
        });

        let response = request_sender.send_request(request).await?;
        if response.status() == 200 {
            Ok(response)
        } else {
            Err(Error::UnprocessableEntity)
        }
    }
}

/// Client that connects to the local tailscaled over TCP with a password. This
/// is used on Windows and macOS when sandboxing is enabled.
pub struct TcpWithPasswordClient {
    port: u16,
    password: String,
}

#[async_trait]
impl LocalApiClient for TcpWithPasswordClient {
    async fn get(&self, uri: Uri) -> Result<Response<Body>> {
        let request = Request::builder()
            .method("GET")
            .header(HOST, "local-tailscaled.sock")
            .header(
                AUTHORIZATION,
                format!(
                    "Basic {}",
                    base64::engine::general_purpose::STANDARD_NO_PAD
                        .encode(format!(":{}", self.password))
                ),
            )
            .uri(uri)
            .body(Body::empty())?;

        let response = self.request(request).await?;
        Ok(response)
    }
}

impl TcpWithPasswordClient {
    async fn request(&self, request: Request<Body>) -> Result<Response<Body>> {
        let stream = TcpSocket::new_v4()?
            .connect((Ipv4Addr::LOCALHOST, self.port).into())
            .await?;
        let (mut request_sender, connection) = hyper::client::conn::handshake(stream).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Error in connection: {}", e);
            }
        });

        let response = request_sender.send_request(request).await?;
        if response.status() == 200 {
            Ok(response)
        } else {
            Err(Error::UnprocessableEntity)
        }
    }
}
