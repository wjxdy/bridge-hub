use std::{
    error::Error,
    io,
    path::PathBuf,
    process::{ExitStatus, Stdio},
    time::Duration,
};

use bridgehub_codex_spike::{app_server_args, handshake};
use clap::Parser;

#[cfg(windows)]
use process_wrap::tokio::JobObject;

#[cfg(unix)]
use process_wrap::tokio::ProcessGroup;
use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    task::JoinHandle,
    time::{Instant, sleep, timeout},
};

use tracing::debug;
use tracing_subscriber::EnvFilter;

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

const EXIT_GRACE_PERIOD: Duration = Duration::from_secs(2);

const EXIT_POLL_INTERVAL: Duration = Duration::from_millis(25);

const STDERR_DRAIN_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Parser)]
#[command(about = "Verify the Codex app-server stdio initialize handshake")]
struct Args {
    #[arg(long, default_value = "codex")]
    codex_bin: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();
    run(Args::parse()).await
}

async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let mut command = CommandWrap::with_new(&args.codex_bin, |command| {
        command
            .args(app_server_args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
    });

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

    let mut reader = BufReader::new(stdout);

    let handshake_result: Result<_, Box<dyn Error>> =
        match timeout(HANDSHAKE_TIMEOUT, handshake(&mut reader, &mut stdin)).await {
            Ok(result) => result.map_err(Into::into),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("Codex initialize handshake exceeded {HANDSHAKE_TIMEOUT:?}"),
            )
            .into()),
        };

    drop(stdin);
    drop(reader);

    let shutdown_result = wait_or_kill(child.as_mut(), EXIT_GRACE_PERIOD).await;
    finish_stderr_task(&mut stderr_task).await;
    if handshake_result.is_err()
        && let Err(error) = &shutdown_result
    {
        debug!(%error, "Codex cleanup also failed after handshake error");
    }

    let initialized = handshake_result?;
    let status = shutdown_result?;

    if !status.success() {
        return Err(format!("Codex app-server exited with {status}").into());
    }

    println!("handshake=ok");
    println!("user_agent={}", initialized.user_agent);
    println!("codex_home={}", initialized.codex_home.display());
    println!("platform_family={}", initialized.platform_family);
    println!("platform_os={}", initialized.platform_os);
    Ok(())
}

fn take_child_pipes(
    child: &mut dyn ChildWrapper,
) -> io::Result<(
    tokio::process::ChildStdin,
    tokio::process::ChildStdout,
    tokio::process::ChildStderr,
)> {
    let stdin = child
        .stdin()
        .take()
        .ok_or_else(|| io::Error::other("Codex child stdin was not piped"))?;

    let stdout = child
        .stdout()
        .take()
        .ok_or_else(|| io::Error::other("Codex child stdout was not piped"))?;

    let stderr = child
        .stderr()
        .take()
        .ok_or_else(|| io::Error::other("Codex child stderr was not piped"))?;

    Ok((stdin, stdout, stderr))
}

async fn wait_or_kill(
    child: &mut dyn ChildWrapper,
    grace_period: Duration,
) -> io::Result<ExitStatus> {
    let deadline = Instant::now() + grace_period;

    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }

        if Instant::now() >= deadline {
            if let Err(kill_error) = child.start_kill() {
                if let Some(status) = child.try_wait()? {
                    return Ok(status);
                }

                return Err(kill_error);
            }

            return child.wait().await;
        }

        sleep(EXIT_POLL_INTERVAL).await;
    }
}

async fn finish_stderr_task(stderr_task: &mut JoinHandle<io::Result<()>>) {
    match timeout(STDERR_DRAIN_TIMEOUT, &mut *stderr_task).await {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(error))) => debug!(%error, "failed while draining Codex stderr"),
        Ok(Err(error)) => debug!(%error, "Codex stderr task failed"),

        Err(_) => {
            stderr_task.abort();
            if let Err(error) = stderr_task.await
                && !error.is_cancelled()
            {
                debug!(%error, "Codex stderr task failed while being stopped");
            }
        }
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("bridgehub_codex_spike=info,codex_app_server=off"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .init();
}
