use std::{error::Error, io, path::PathBuf, process::Stdio, time::Duration};

use bridgehub_codex_spike::{app_server_args, handshake};
use clap::Parser;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    time::timeout,
};
use tracing::debug;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(about = "Verify the Codex app-server stdio initialize handshake")]
struct Args {
    #[arg(long, default_value = "codex")]
    codex_bin: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();
    let args = Args::parse();
    let mut child = Command::new(&args.codex_bin)
        .args(app_server_args())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("Codex child stdin was not piped"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Codex child stdout was not piped"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("Codex child stderr was not piped"))?;

    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Some(line) = lines.next_line().await? {
            debug!(target: "codex_app_server", message = %line);
        }
        Ok::<(), io::Error>(())
    });

    let mut reader = BufReader::new(stdout);
    let initialized = handshake(&mut reader, &mut stdin).await?;
    println!("handshake=ok");
    println!("user_agent={}", initialized.user_agent);
    println!("codex_home={}", initialized.codex_home.display());
    println!("platform_family={}", initialized.platform_family);
    println!("platform_os={}", initialized.platform_os);

    drop(stdin);
    let status = match timeout(Duration::from_secs(2), child.wait()).await {
        Ok(wait_result) => wait_result?,
        Err(_) => {
            child.kill().await?;
            child.wait().await?
        }
    };
    match stderr_task.await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => debug!(%error, "failed while draining Codex stderr"),
        Err(error) => debug!(%error, "Codex stderr task failed"),
    }
    if !status.success() {
        return Err(format!("Codex app-server exited with {status}").into());
    }
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("bridgehub_codex_spike=info,codex_app_server=off"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
