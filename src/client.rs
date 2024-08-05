
use std::ffi::OsString;
use std::time::Duration;
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::{join, time};
use windows::Win32::Foundation::{ERROR_NOT_FOUND, ERROR_PIPE_BUSY};

async fn run(pipe_path: OsString) -> anyhow::Result<()> {
    // TODO: Add a timeout for not found
    let client = loop {
        match ClientOptions::new().open(&pipe_path) {
            Ok(client) => break client,
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) => (),
            Err(e) if e.raw_os_error() == Some(ERROR_NOT_FOUND.0 as i32) => (),
            Err(e) => return Err(e.into()),
        }
    
        time::sleep(Duration::from_millis(50)).await;
    };

    let (mut client_reader, mut client_writer) = tokio::io::split(client);
    let mut stdout = tokio::io::stdout();
    let mut stdin = tokio::io::stdin();
    
    let input_future = tokio::spawn(async move {
        tokio::io::copy(&mut client_reader, &mut stdout).await
    });
    let output_future = tokio::spawn(async move {
        tokio::io::copy(&mut stdin, &mut client_writer).await
    });

    let _ = join!(input_future, output_future);

    Ok(())
}

pub fn attach(pipe_path: &str) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(run(pipe_path.into()))?;
    Ok(())
}
