use anyhow::Result;
use std::process::Command;

/// Represents parameters for a command execution
#[derive(Debug, Clone)]
pub struct CommandParams {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: Option<String>,
}

impl CommandParams {
    pub fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            current_dir: None,
        }
    }
}

/// Handles execution of commands with dry run capabilities
pub struct CommandExecutor {
    pub dry_run: bool,
}

impl CommandExecutor {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// Execute a command with the provided parameters
    pub fn execute(&self, params: &CommandParams) -> Result<()> {
        if self.dry_run {
            let args_str = params.args.join(" ");
            if let Some(ref dir) = params.current_dir {
                println!("  Command: (cd {} && {} {})", dir, params.program, args_str);
            } else {
                println!("  Command: {} {}", params.program, args_str);
            }
            return Ok(());
        }

        let mut command = Command::new(&params.program);
        
        command.args(&params.args);
        
        if let Some(ref dir) = params.current_dir {
            command.current_dir(dir);
        }
        
        let result = command.status()?;
        
        if !result.success() {
            return Err(anyhow::anyhow!(
                "Command '{}' failed with exit code: {:?}",
                params.program,
                result.code()
            ));
        }

        Ok(())
    }

    /// Execute a command with the provided parameters and capture the output
    pub fn execute_with_output(&self, params: &CommandParams) -> Result<String> {
        if self.dry_run {
            let args_str = params.args.join(" ");
            if let Some(ref dir) = params.current_dir {
                println!("  Command (with output): (cd {} && {} {})", dir, params.program, args_str);
            } else {
                println!("  Command (with output): {} {}", params.program, args_str);
            }
            // Return a mock output for dry run
            return Ok("mock output for dry run".to_string());
        }

        let mut command = Command::new(&params.program);
        
        command.args(&params.args);
        
        if let Some(ref dir) = params.current_dir {
            command.current_dir(dir);
        }
        
        let output = command.output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Command '{}' failed with exit code: {:?}",
                params.program,
                output.status.code()
            ));
        }

        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout.trim().to_string())
    }
}