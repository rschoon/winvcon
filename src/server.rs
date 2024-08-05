
use anyhow::Context;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::mem;
use std::sync::Arc;
use tokio::join;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::sync::{broadcast, mpsc};

use crate::win::command::{current_process_path, Command, ProcessHandle};
use crate::win::console::PseudoConsole;

pub fn launch_in_background(_path: &str) -> anyhow::Result<()> {
    let exe_path = current_process_path();

    Command::new(exe_path)
        .arg("server-launch")
        .spawn_into_background()?;

    Ok(())
}

struct Server {
    pipe_path: OsString,
    console: PseudoConsole,
    input_tx: mpsc::Sender<InputCommand>,
    output_tx: broadcast::Sender<OutputCommand>,
    proc: ProcessHandle
}

impl Server {
    async fn serve_forever(self: Arc<Self>) -> anyhow::Result<()> {
        let mut server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&self.pipe_path)?;

        loop {
            server.connect().await?;
            let connected_client = server;
            server = ServerOptions::new().create(&self.pipe_path)?;

            let this = self.clone();
            tokio::spawn(this.handle_client(connected_client));
        }
    }

    async fn handle_client(self: Arc<Self>, client: NamedPipeServer) {
        let (mut client_read, mut client_write) = tokio::io::split(client);

        let mut output_rx = self.output_tx.subscribe();
        let output_future = async {
            while let Ok(cmd) = output_rx.recv().await {
                if client_write.write_all(&cmd.0).await.is_err() {
                    break;
                }
            }
        };

        let input_future = async {
            let mut buf = vec![0u8; 256];
            while let Ok(sz) = client_read.read(&mut buf).await {
                buf.truncate(sz);
                if self.input_tx.send(InputCommand(mem::take(&mut buf))).await.is_err() {
                    break;
                }
                buf = vec![0u8; 256];
            }
        };

        join!(input_future, output_future);
    }

    fn pump_input(self: Arc<Self>, mut input_rx: mpsc::Receiver<InputCommand>) {
        let pipe = self.console.stdin();
        while let Some(cmd) = input_rx.blocking_recv() {
            if pipe.as_ref().write_all(&cmd.0).is_err() {
                break;
            }
        }
    }

    fn pump_output(self: Arc<Self>) {
        let pipe = self.console.stdout();
        let mut buf = Vec::with_capacity(256);
        while let Ok(sz) = pipe.as_ref().read(&mut buf) {
            buf.truncate(sz);
            let _ = self.output_tx.send(OutputCommand(Arc::new(mem::take(&mut buf))));
            buf = vec![0u8; 256];
        }
    }
}

#[derive(Clone)]
struct InputCommand(Vec<u8>);

#[derive(Clone)]
struct OutputCommand(Arc<Vec<u8>>);

async fn run(server: Arc<Server>, input_rx: mpsc::Receiver<InputCommand>) -> anyhow::Result<()> {
    tokio::task::spawn_blocking({
        let server = server.clone();
        || server.pump_input(input_rx)
    });

    tokio::task::spawn_blocking({
        let server = server.clone();
        || server.pump_output()
    });

    tokio::spawn(server.clone().serve_forever());
    
    tokio::task::spawn_blocking(move || {
        server.proc.wait();
    }).await?;

    Ok(())
}

pub fn main(pipe_path: &str) -> anyhow::Result<()> {
    let console = PseudoConsole::new(80, 24).with_context(|| "Failed to create console")?;
    let command = Command::new("cmd.exe");
    let proc = command.spawn_into(&console).with_context(|| "Failed to spawn process")?;

    let (input_tx, input_rx) = mpsc::channel(128);
    let output_tx = broadcast::Sender::new(128);

    let server = Arc::new(Server {
        pipe_path: OsString::from(pipe_path),
        console,
        input_tx,
        output_tx,
        proc
    });

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(run(server, input_rx))?;

    Ok(())
}
