use std::path::Path;
use std::process::{Command, Output};

use log::warn;

use super::{AgentInjector, InjectError};

#[derive(Debug, Default)]
pub struct PlatformInjector;

impl AgentInjector for PlatformInjector {
    fn is_agent_loaded(&self, pid: u32, agent_file: &str) -> bool {
        match run_jcmd(pid, "VM.dynlibs", None) {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).contains(agent_file)
            }
            Ok(output) => {
                warn!(
                    "could not inspect loaded libraries in process {pid}: {}",
                    command_output(&output)
                );
                false
            }
            Err(error) => {
                warn!("could not inspect loaded libraries in process {pid}: {error}");
                false
            }
        }
    }

    fn inject(&self, pid: u32, agent_path: &Path) -> Result<(), InjectError> {
        let absolute_path = std::path::absolute(agent_path).map_err(InjectError::Path)?;
        let output = run_jcmd(pid, "JVMTI.agent_load", Some(&absolute_path))?;
        if !output.status.success() {
            return Err(InjectError::Inject(
                format!(
                    "jcmd could not load the agent into process {pid}: {}",
                    command_output(&output)
                )
                .into(),
            ));
        }
        Ok(())
    }
}

fn run_jcmd(pid: u32, operation: &str, library_path: Option<&Path>) -> Result<Output, InjectError> {
    let mut command = Command::new(jcmd_path());
    command.arg(pid.to_string()).arg(operation);
    if let Some(path) = library_path {
        command.arg(path);
    }
    command.output().map_err(|error| {
        InjectError::Inject(format!("could not run jcmd for process {pid}: {error}").into())
    })
}

fn jcmd_path() -> std::ffi::OsString {
    if let Some(home) = std::env::var_os("JAVA_HOME") {
        let candidate = Path::new(&home).join("bin/jcmd");
        if candidate.is_file() {
            return candidate.into_os_string();
        }
    }
    "jcmd".into()
}

fn command_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    match (stdout.trim(), stderr.trim()) {
        ("", "") => format!("exit status {}", output.status),
        ("", stderr) => stderr.to_string(),
        (stdout, "") => stdout.to_string(),
        (stdout, stderr) => format!("{stdout}\n{stderr}"),
    }
}

pub fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}
