
pub mod win;

fn main() -> anyhow::Result<()> {
    let console = win::console::PseudoConsole::new(80, 24)?;

    let command = win::command::Command::new("cmd.exe");
    command.spawn_into(&console)?;

    Ok(())
}
