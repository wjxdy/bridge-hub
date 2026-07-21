pub const fn client_name() -> &'static str {
    "bridgehub"
}

#[cfg(test)]
mod tests {

    use std::error::Error;

    use serde_json::{Value, json};
use tokio::io::BufWriter;

    use super::{InitalizeResponse, handshake};

    #[test]
    fn client_identity_is_stable() {
        assert_eq!(super::client_name(), "bridgehub");
    }

    #[tokio::test]
    async fn handshake_sends_initialize_then_initialized(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {

        let (client_io, server_io) = tokio::io::duplex(8 * 1024);

        let (client_reader, mut client_writer) = tokio::io::split(client_io);

        let (server_reader, mut server_writer) = tokio::io::split(server_io);

        let fake_server = tokio::spawn(async move {
            
            let mut reader = BufWriter::new(server_writer);

            let mut initialize_line = String::new();

            reader.read_line(&mut initialize_line).await?;

            let initialize: Value = serde_json::from_json

        });

    }

}
