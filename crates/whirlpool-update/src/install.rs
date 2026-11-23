use std::{fmt, path::Path, process::Command};

#[derive(Debug)]
pub struct InstallError(pub String);

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not start Setup: {}", self.0)
    }
}

/// Starts the verified installer detached and returns at once. The caller exits right after, so
/// Setup can replace `whirlpool.exe`. Setup stops a running dock itself, installs over the
/// current copy keeping the options chosen last time, and starts Whirlpool again (`/relaunch=1`).
pub fn install(installer: &Path, quiet: bool) -> Result<(), InstallError> {
    let mut command = Command::new(installer);
    command.args([
        if quiet { "/VERYSILENT" } else { "/SILENT" },
        "/SUPPRESSMSGBOXES",
        "/NORESTART",
        "/CLOSEAPPLICATIONS",
        "/relaunch=1",
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| InstallError(error.to_string()))
}
