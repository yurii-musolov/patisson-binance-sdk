//! WebSocket connections through an HTTP proxy (`CONNECT` tunnel).
//!
//! tungstenite has no proxy support: open a TCP connection to the proxy, ask
//! it to `CONNECT` to the Binance host, then run TLS and the WebSocket
//! handshake over the tunnel.

use reqwest::Url;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, client_async_tls,
    tungstenite::{Error, error::UrlError, handshake::client::Response},
};

/// Largest proxy response header we accept before giving up.
const MAX_PROXY_RESPONSE: usize = 16 * 1024;

pub(crate) async fn connect_via_proxy(
    url: &str,
    proxy: &str,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error> {
    let target = Url::parse(url).map_err(|_| Error::Url(UrlError::NoHostName))?;
    let host = target.host_str().ok_or(Error::Url(UrlError::NoHostName))?;
    let port = target
        .port_or_known_default()
        .ok_or(Error::Url(UrlError::UnsupportedUrlScheme))?;

    let proxy = Url::parse(proxy).map_err(|_| Error::Url(UrlError::NoHostName))?;
    if proxy.scheme() != "http" {
        return Err(Error::Url(UrlError::UnsupportedUrlScheme));
    }
    let proxy_host = proxy.host_str().ok_or(Error::Url(UrlError::NoHostName))?;
    let proxy_port = proxy.port_or_known_default().unwrap_or(80);

    let mut tcp = TcpStream::connect((proxy_host, proxy_port)).await?;
    let mut request = format!("CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\n");
    if !proxy.username().is_empty() {
        let credentials = format!(
            "{}:{}",
            percent_decode(proxy.username()),
            percent_decode(proxy.password().unwrap_or(""))
        );
        request.push_str(&format!(
            "Proxy-Authorization: Basic {}\r\n",
            base64(credentials.as_bytes())
        ));
    }
    request.push_str("\r\n");
    tcp.write_all(request.as_bytes()).await?;

    // Read the proxy's response header (up to the blank line). Nothing
    // follows it until we start the TLS handshake, so reading byte-wise
    // cannot swallow tunnelled data.
    let mut header = Vec::with_capacity(256);
    while !header.ends_with(b"\r\n\r\n") {
        if header.len() >= MAX_PROXY_RESPONSE {
            return Err(proxy_error("response header too large"));
        }
        let byte = tcp.read_u8().await?;
        header.push(byte);
    }
    let status_line = header
        .split(|&b| b == b'\n')
        .next()
        .map(|line| String::from_utf8_lossy(line).trim().to_owned())
        .unwrap_or_default();
    let status = status_line.split_whitespace().nth(1).unwrap_or_default();
    if status != "200" {
        return Err(proxy_error(&format!("CONNECT refused: {status_line}")));
    }

    client_async_tls(url, tcp).await
}

fn proxy_error(msg: &str) -> Error {
    Error::Io(std::io::Error::other(format!("proxy: {msg}")))
}

/// Decode `%XX` escapes of URL user info.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Some(byte) = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Standard base64 with padding (for `Proxy-Authorization: Basic`).
fn base64(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            if i <= chunk.len() {
                out.push(ALPHABET[((n >> shift) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc4648_vectors() {
        for (input, expected) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
            ("user:p@ss", "dXNlcjpwQHNz"),
        ] {
            assert_eq!(base64(input.as_bytes()), expected, "{input:?}");
        }
    }

    #[test]
    fn percent_decoding_of_user_info() {
        assert_eq!(percent_decode("p%40ss%3Aword"), "p@ss:word");
        assert_eq!(percent_decode("plain"), "plain");
        assert_eq!(percent_decode("bad%zz"), "bad%zz");
        assert_eq!(percent_decode("trail%4"), "trail%4");
    }
}
