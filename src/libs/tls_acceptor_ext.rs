use std::{path::Path, sync::Arc};

use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use tokio_rustls::TlsAcceptor;

static CERT_FILE_NAME: &'static str = "cert.pem";
static KEY_FILE_NAME: &'static str = "key.pem";

pub(crate) trait TLSAcceptorLoader {
    async fn load(absolute_mocks_path: impl AsRef<Path>) -> Option<TlsAcceptor>;
}

impl TLSAcceptorLoader for TlsAcceptor {
    async fn load(absolute_mocks_path: impl AsRef<Path>) -> Option<TlsAcceptor> {
        let cert_path = absolute_mocks_path.as_ref().join(CERT_FILE_NAME);
        let key_path = absolute_mocks_path.as_ref().join(KEY_FILE_NAME);

        let (cert, key) = tokio::join!(get_cert(cert_path), get_key(key_path));

        cert.zip(key)
            .and_then(|(cert, key)| {
                ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(cert, key)
                    .ok()
            })
            .map(|config| TlsAcceptor::from(Arc::new(config)))
    }
}

async fn get_key(path: impl AsRef<Path>) -> Option<PrivateKeyDer<'static>> {
    let path = Arc::new(path.as_ref().to_owned());
    let (p1, p2) = (path.clone(), path.clone());

    let pkcs8 = tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(p1.as_ref()).ok()?;
        let mut reader = std::io::BufReader::new(file);
        let mut keys = pkcs8_private_keys(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        let key: Option<PrivateKeyDer<'static>> = keys.pop().map(|v| v.into());

        key
    })
    .await
    .ok()
    .flatten();

    if let Some(key) = pkcs8 {
        return Some(key);
    }

    let rsa = tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(p2.as_ref()).ok()?;
        let mut reader = std::io::BufReader::new(file);
        let mut keys = rsa_private_keys(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        let key: Option<PrivateKeyDer<'static>> = keys.pop().map(|v| v.into());

        key
    })
    .await
    .ok()
    .flatten();

    if let Some(key) = rsa {
        return Some(key);
    }

    None
}

async fn get_cert(path: impl AsRef<Path>) -> Option<Vec<CertificateDer<'static>>> {
    let path = path.as_ref().to_owned();
    tokio::task::spawn_blocking(|| {
        let file = std::fs::File::open(path).ok()?;
        let mut buf_reader = std::io::BufReader::new(file);
        let certs = certs(&mut buf_reader).collect::<Result<Vec<_>, _>>().ok()?;

        Some(certs)
    })
    .await
    .ok()
    .flatten()
    .filter(|v| !v.is_empty())
}
