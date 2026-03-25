use anyhow::{Context, Result, anyhow};
use std::path::Path;
use std::process::{Command, Output};

pub fn run_command(program: &str, args: &[&str], cwd: Option<&Path>) -> Result<Output> {
    let mut command = Command::new(program);
    command.args(args);

    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let output = command
        .output()
        .with_context(|| format!("failed to run command: {} {}", program, args.join(" ")))?;

    if output.status.success() {
        Ok(output)
    } else {
        Err(anyhow!(
            "command failed: {} {}\nstatus: {:?}\nstdout: {}\nstderr: {}",
            program,
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
