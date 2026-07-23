use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite};

pub const fn client_name() -> &'static str {
    "bridgehub"
}

#[derive(Debug, Error)]
pub enum HandShakeError {
    #[error("app-server closed before initialize completed")]
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
    pub platfrom_family: String,
    pub platfrom_os: String,
}

struct Request<T> {
    method: &'static str,
    id: u64,
    params: T,
}

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
) -> Result<InitializeResponse, HandShakeError>
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
        return Err(HandShakeError::UnexpectedEof);
    }

    let envelope: ResponseEnvelope = serde_json::from_str(&line)?;

    if envelope.id != 0 {
        return Err(HandShakeError::UnexpectedResponseId {
            actual: envelope.id,
        });
    }
}

#[cfg(test)]
mod tests {

    use std::error::Error;

    use serde_json::{Value, json};
    use tokio::io::{AsyncWriteExt, BufReader, BufWriter};

    use super::{InitalizeResponse, handshake};

    #[test]
    fn client_identity_is_stable() {
        assert_eq!(super::client_name(), "bridgehub");
    }

    #[tokio::test]
    async fn handshake_sends_initialize_then_initialized()
    -> Result<(), Box<dyn Error + Send + Sync>> {
        let (client_io, server_io) = tokio::io::duplex(8 * 1024);

        let (client_reader, mut client_writer) = tokio::io::split(client_io);

        let (server_reader, mut server_writer) = tokio::io::split(server_io);

        let fake_server = tokio::spawn(async move {
            let mut reader = BufWriter::new(server_reader);

            let mut initialize_line = String::new();

            reader.readline(&mut initialize_line).await?;

            let initialize: Value = serde_json::from_str(&initialize_line)?;

            assert_eq!(initialize["method"], "initialize");

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
                    "platfromFamily": "unix",
                    "platfromfromOs": "macos"
                }
            });

            server_writer
                .write_all(response.to_string().as_bytes())
                .await?;

            server_writer.write_all(b"\n").await?;

            server_writer.flush().await?;

            let mut initialized_line = String::new();

            reader.read_line(&mut initialize_line).await?;

            let initialized: Value = serde_json::from_str(&initialize_line)?;

            assert_eq!(initialized, json!({ "method": "initialized" }));

            Ok::<(), Box<dyn Error + Send + Sync>>(())
        });

        let mut reader = BufReader::new(client_reader);

        let result = handshake(&mut reader, &mut client_writer).await?;

        assert_eq!(
            result,
            InitalizeResponse {
                user_agent: "codex-cli_rs/test".to_owned(),
                codex_home: "/tmp/codex-home".into(),
                platfrom_family: "unix".to_owned(),
                platfrom_os: "macos".to_owned(),
            }
        );

        fake_server.await?;

        Ok(())
    }
}
