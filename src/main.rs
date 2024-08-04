use anyhow::Context;


pub mod win;

fn main() -> anyhow::Result<()> {
    let console = win::console::PseudoConsole::new(80, 24).with_context(|| "Failed to create console")?;

    let command = win::command::Command::new("cmd.exe");
    command.spawn_into(&console).with_context(|| "Failed to spawn process")?;

    Ok(())
}
