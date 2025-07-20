use std::path::Path;
use anyhow::Result;
use crate::is_power_shell;

pub fn create_shim<P: AsRef<Path>>(
    target: P,
    shim_path: P
) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(target, shim_path)?;
    }
    #[cfg(windows)]
    {
        use std::fs::write;
        let script = format!(
            "@echo off\r\ncall \"{}\" %*\r\n",
            target.as_ref().display()
        );
        write(shim_path.as_ref().with_extension("bat"), script)?;
    }
    Ok(())
}

#[cfg(target_family = "unix")]
pub fn write_unix_activate() -> std::io::Result<()> {
    let content = r#"#!/bin/sh
    export PATH="$(pwd)/.frate/shims:$PATH"
    echo "Frate shell activated. Type 'exit' to leave."
    exec "$SHELL"
    "#;

    std::fs::write("./.frate/activate", content)?;
    std::process::Command::new("chmod")
        .arg("+x")
        .arg("./.frate/activate")
        .status()?; // safer than `.output()` here
    Ok(())
}

#[cfg(target_family = "windows")]
pub fn write_windows_activate() -> std::io::Result<()> {
    let shim_path = r#"%CD%\.frate\shims"#;

    if is_power_shell() {
        let content = format!(
            r#"$env:PATH = "{};${{env:PATH}}"
            Write-Host "Frate shell activated. Type 'exit' to leave.""#,
            shim_path
        );
        std::fs::write(".frate\\activate.ps1", content)?;
    } else {
        let content = format!(
            r#"@echo off
            set "PATH={};%PATH%"
            echo Frate shell activated. Type "exit" to leave.
            cmd
            "#,
            shim_path
        );
        std::fs::write(".frate\\activate.bat", content)?;
    }
    Ok(())
}


pub fn run_shell_with_frate_path() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use std::process::Command;

    let frate_shims = format!(
        "{}\\.frate\\shims",
        std::env::current_dir()?.display()
    );

    let path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{frate_shims};{path}");

    // Check if PowerShell exists
    let powershell = if Command::new("pwsh").output().is_ok() {
        "pwsh"
    } else {
        "powershell"
    };

    Command::new(powershell)
        .args(&[
            "-NoExit",
            "-Command",
            &format!(
                "$env:PATH='{}'; Write-Host 'Frate shell activated. Type \"exit\" to leave.'",
                new_path.replace('\\', "\\\\") // Escape backslashes
            ),
        ])
        .spawn()?
        .wait()?;
    }

    #[cfg(unix)]
    {
        use std::process::Command;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let frate_shims = format!("{}/.frate/shims", std::env::current_dir()?.display());
        let current_path = std::env::var("PATH").unwrap_or_default();
        let new_path = format!("{frate_shims}:{}", current_path);

        Command::new(&shell)
            .arg("-i")
            .env("PATH", new_path)
            .spawn()?
            .wait()?;
    }

    Ok(())
}
