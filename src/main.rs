
use clap::{Args, Parser, Subcommand};

pub mod win;
mod server;
mod client;

static PIPE_PATH: &str = r"\\.\pipe\winvcon-default";

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Create(CreateArgs),
    Attach(AttachArgs),
    #[clap(hide = true)]
    ServerLaunch(ServerLaunchArgs),
}

#[derive(Debug, Args)]
struct CreateArgs {

}

#[derive(Debug, Args)]
struct AttachArgs {

}

#[derive(Debug, Args)]
struct ServerLaunchArgs {

}

fn create(_args: CreateArgs) -> anyhow::Result<()> {
    server::launch_in_background(PIPE_PATH)?;
    client::attach(PIPE_PATH)
}

fn attach(_args: AttachArgs) -> anyhow::Result<()> {
    client::attach(PIPE_PATH)
}

fn launch(_args: ServerLaunchArgs) -> anyhow::Result<()> {
    server::main(PIPE_PATH)
}

fn main() -> anyhow::Result<()> {

    let args = Cli::parse();
    match args.command {
        Command::Create(args) => create(args)?,
        Command::Attach(args) => attach(args)?,
        Command::ServerLaunch(args) => launch(args)?,
    }

    Ok(())
}
