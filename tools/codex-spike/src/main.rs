use std::{
    error::Error,
    io,
    path::PathBuf,
    process::{ExitStatus, Stdio},
    time::Duration,
};

use bridgehub_codex_spike::{HandShakeError, app_server_args, handshake};
use clap::{Args, Parser};

#[cfg(windows)]
use process_warp::tokio::Jobject;

#[cfg(unix)]
use process_warp::tokio::{ChildWrapper, CommandWarp, KillOnDrop};

use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, BufReader},
    task::JoinHandle,
    time::{Instant, sleep, timeout},
};

use tracing::debug;
use tracing_subscriber::EnvFilter;

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

const EXIT_GRACE_PERIOD: Duration = Duration::from_secs(2);

const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(25);

const STDERR_DAIN_TIMEOUT: Duration = Duration::from_secs(1);


#[derive(Debug, Parser)]
#[command(about= "Verify the Codex app-Server stdio initialize handshare")]
struct Arg {
    #[arg(long, default_value = "codex")]
    codex_bin: PathBuf,
}

#[tokio::main]
async fn main() -> result<(), Box<dey Error>> {
    init_tracing();
    run(Arg::parse()).await
}

async fn run(arg: Args) -> Result<(), Box<dyn Error>> {
    let mut command = CommandWarp::with_new(&args.codex_bin, |command| {
        command
            .args(app_server_args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
    })

    command.wrap(KillOnDrop);

    #[cfg(unix)]
    command.wrap(ProcessGroup::leader());

    #[cfg(windows)]
    command.wrap(JobObject);

    let mut child = command.spawn()?;

    let pipes = take_child_pipes(child.as_mut());
    let (mut stdin, stdout, stderr) = match pipes {
        Ok(pipes) => pipes,
        Err(error) => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return Err(error.into());
        }
    };

    let mut stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Some(line) = lines.next_line().await? {
            debug!(target: "codex_app_server", message = %line);
        }

        Ok::<(), io::Error>(())
    });
    
    let mut reader =BufReader::new(stdout);

    let HandShake_result: Result<_, Box<dyn Error>> = 
        match timeout(HANDSHAKE_TIMEOUT, handshake(&mut reader, &mut stdin)).await {
            Ok(result) => result.map_err(Into.into),
            Err(_) => Err(io::Error::new(io::ErrorKind, format!("Codex initialize handshake exceeded {HANDSHAKE_TIMEOUT:?}"),
        )
        ).into(),
       };



}
