use std::path::Path;
use anyhow::Result;
use crate::is_power_shell;

/// Creates a platform-specific "shim" to forward execution to a target binary.
///
/// On Unix systems, this creates a symbolic link (`symlink`) at `shim_path` pointing to `target`.
/// On Windows, it creates a `.bat` script at `shim_path` (with a `.bat` extension) that calls the `target`.
///
/// # Arguments
///
/// * `target` - Path to the executable or script to forward to.
/// * `shim_path` - Path where the shim (symlink or .bat) will be created.
///
/// # Errors
///
/// Returns an error if the symlink (on Unix) or file write (on Windows) fails.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use frate::create_shim;
///
/// let target = PathBuf::from("/usr/bin/python3");
/// let shim = PathBuf::from("./.frate/shims/python");
/// create_shim(target, shim).unwrap();
/// ```
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


/// Writes a Unix shell script to activate the Frate environment.
///
/// The script sets the `PATH` to include the `./.frate/shims` directory,
/// prints a message, and launches a new interactive shell.
///
/// The script is written to `./.frate/activate` and marked as executable.
///
/// # Errors
///
/// Returns an error if writing the file or setting the executable bit fails.
///
/// # Example
///
/// ```no_run
/// #[cfg(target_family = "unix")]
/// {
/// use frate::write_unix_activate;
/// write_unix_activate().unwrap();
/// }
/// ```
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

/// Writes a Windows batch or PowerShell script to activate the Frate environment.
///
/// If PowerShell is detected, a `.ps1` script is written. Otherwise, a `.bat` file is created.
/// Both scripts add `.\.frate\shims` to the `PATH` and show an activation message.
///
/// # Errors
///
/// Returns an error if writing the activation script fails.
///
/// # Example
///
/// ```no_run
/// #[cfg(target_family = "windows")]
/// {
/// use frate::write_windows_activate;
/// write_windows_activate().unwrap();
/// }
/// ```
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

/// Starts a new interactive shell with the Frate shims path prepended to the `PATH`.
///
/// On Windows:
/// - Attempts to launch PowerShell or fallback to `powershell.exe`.
/// - Prepends `.frate\shims` to the `PATH` and starts a session with a message.
///
/// On Unix:
/// - Uses the shell defined in `$SHELL` or falls back to `/bin/sh`.
/// - Prepends `.frate/shims` to the `PATH` and starts an interactive shell.
///
/// # Errors
///
/// Returns an error if:
/// - The current working directory cannot be determined.
/// - The shell process cannot be spawned or fails.
///
/// # Example
///
/// ```no_run
/// use frate::run_shell_with_frate_path;
/// run_shell_with_frate_path().unwrap();
/// ```

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
