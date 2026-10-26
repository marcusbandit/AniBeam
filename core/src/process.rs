//! Starting the helpers the core runs (anirss, qBittorrent, an endpoint's
//! start command) the same way on every desktop. On Windows a child gets no
//! console window, and the anirss a package carries is a Python script, so it
//! runs through the Python the package carries beside it.

use tokio::process::Command;

/// Set by a Windows shell whose package carries anirss's Python.
#[cfg(windows)]
const PYTHON_ENV: &str = "ANIBEAM_PYTHON";

/// CREATE_NO_WINDOW, from WinBase.h.
#[cfg(windows)]
pub(crate) const NO_WINDOW: u32 = 0x0800_0000;

/// A command for `program`. Everywhere but Windows this is `Command::new`.
pub(crate) fn command(program: &str) -> Command {
    #[cfg(windows)]
    {
        let python = std::env::var_os(PYTHON_ENV).filter(|_| is_script(program));
        let mut cmd = match python {
            Some(python) => {
                let mut cmd = Command::new(python);
                cmd.arg(program).env("PYTHONUTF8", "1");
                cmd
            }
            None => Command::new(program),
        };
        cmd.creation_flags(NO_WINDOW);
        cmd
    }
    #[cfg(not(windows))]
    Command::new(program)
}

/// A file with no extension, which Windows cannot start by itself: the
/// bundled anirss script.
#[cfg(windows)]
fn is_script(program: &str) -> bool {
    let path = std::path::Path::new(program);
    path.extension().is_none() && path.is_file()
}

/// A shell line, as an endpoint's start command is written: `sh -c`, or on
/// Windows `cmd /C`.
pub(crate) fn shell(line: &str) -> Command {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").raw_arg(line).creation_flags(NO_WINDOW);
        cmd
    }
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(line);
        cmd
    }
}
