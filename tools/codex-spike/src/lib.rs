use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

pub const fn client_name() -> &'static str {
    "bridgehub"
}

pub const fn app_server_args() -> [&'static str; 3] {
    ["app-server", "--listen", "stdio://"]
}

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("app-server closed stdout before initialize completed")]
    UnexpectedEof,
    #[error("initialize response id was {actual}, expected 0")]
    UnexpectedResponseId { actual: u64 },
    #[error("initialize response did not contain result or error")]
    MissingResult,
    #[error("app-server rejected initialize with {code}: {message}")]
    Server { code: i64, message: String },
    #[error("stdio I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON from app-server: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub user_agent: String,
    pub codex_home: PathBuf,
    pub platform_family: String,
    pub platform_os: String,
}

#[derive(Debug, Serialize)]
struct Request<T> {
    method: &'static str,
    id: u64,
    params: T,
}

#[derive(Debug, Serialize)]
struct Notification {
    method: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    client_info: ClientInfo,
    capabilities: InitializeCapabilities,
}

#[derive(Debug, Serialize)]
struct ClientInfo {
    name: &'static str,
    title: &'static str,
    version: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InitializeCapabilities {
    experimental_api: bool,
    request_attestation: bool,
}

#[derive(Debug, Deserialize)]
struct ResponseEnvelope {
    id: u64,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

pub async fn handshake<R, W>(
    reader: &mut R,
    writer: &mut W,
) -> Result<InitializeResponse, HandshakeError>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let request = Request {
        method: "initialize",
        id: 0,
        params: InitializeParams {
            client_info: ClientInfo {
                name: client_name(),
                title: "BridgeHub",
                version: env!("CARGO_PKG_VERSION"),
            },
            capabilities: InitializeCapabilities {
                experimental_api: false,
                request_attestation: false,
            },
        },
    };
    write_json_line(writer, &request).await?;

    let mut line = String::new();
    if reader.read_line(&mut line).await? == 0 {
        return Err(HandshakeError::UnexpectedEof);
    }
    let envelope: ResponseEnvelope = serde_json::from_str(&line)?;
    if envelope.id != 0 {
        return Err(HandshakeError::UnexpectedResponseId {
            actual: envelope.id,
        });
    }
    if let Some(error) = envelope.error {
        return Err(HandshakeError::Server {
            code: error.code,
            message: error.message,
        });
    }
    let result = envelope.result.ok_or(HandshakeError::MissingResult)?;
    let response = serde_json::from_value(result)?;

    write_json_line(
        writer,
        &Notification {
            method: "initialized",
        },
    )
    .await?;
    Ok(response)
}

async fn write_json_line<W, T>(writer: &mut W, value: &T) -> Result<(), HandshakeError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let mut json = serde_json::to_vec(value)?;
    json.push(b'\n');
    writer.write_all(&json).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use serde_json::{Value, json};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    use super::{InitializeResponse, handshake};

    #[test]
    fn client_identity_is_stable() {
        assert_eq!(super::client_name(), "bridgehub");
    }

    #[test]
    fn app_server_arguments_use_the_supported_stdio_transport() {
        assert_eq!(
            super::app_server_args(),
            ["app-server", "--listen", "stdio://"]
        );
    }

    #[tokio::test]
    async fn handshake_sends_initialize_then_initialized()
    -> Result<(), Box<dyn Error + Send + Sync>> {
        let (client_io, server_io) = tokio::io::duplex(8 * 1024);
        let (client_reader, mut client_writer) = tokio::io::split(client_io);
        let (server_reader, mut server_writer) = tokio::io::split(server_io);

        let fake_server = tokio::spawn(async move {
            let mut reader = BufReader::new(server_reader);
            let mut initialize_line = String::new();
            reader.read_line(&mut initialize_line).await?;
            let initialize: Value = serde_json::from_str(&initialize_line)?;
            assert_eq!(initialize["method"], "initialize");
            assert_eq!(initialize["id"], 0);
            assert_eq!(initialize["params"]["clientInfo"]["name"], "bridgehub");
            assert_eq!(
                initialize["params"]["capabilities"]["experimentalApi"].as_bool(),
                Some(false)
            );

            let response = json!({
                "id": 0,
                "result": {
                    "userAgent": "codex_cli_rs/test",
                    "codexHome": "/tmp/codex-home",
                    "platformFamily": "unix",
                    "platformOs": "macos"
                }
            });
            server_writer
                .write_all(response.to_string().as_bytes())
                .await?;
            server_writer.write_all(b"\n").await?;
            server_writer.flush().await?;

            let mut initialized_line = String::new();
            reader.read_line(&mut initialized_line).await?;
            let initialized: Value = serde_json::from_str(&initialized_line)?;
            assert_eq!(initialized, json!({ "method": "initialized" }));
            Ok::<(), Box<dyn Error + Send + Sync>>(())
        });

        let mut reader = BufReader::new(client_reader);
        let result = handshake(&mut reader, &mut client_writer).await?;
        assert_eq!(
            result,
            InitializeResponse {
                user_agent: "codex_cli_rs/test".to_owned(),
                codex_home: "/tmp/codex-home".into(),
                platform_family: "unix".to_owned(),
                platform_os: "macos".to_owned(),
            }
        );
        fake_server.await??;
        Ok(())
    }
}
