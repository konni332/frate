use std::path::Path;
use anyhow::Result;
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