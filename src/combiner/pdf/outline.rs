use crate::config::combine::OutlineParam;
use anyhow::anyhow;
use std::{fs, path::Path, process::Command};

/// call outline bin to add outlines into target pdf
pub(crate) fn add_outline(
    workspace: &Path,
    param: &OutlineParam,
    bin: &Path,
) -> anyhow::Result<()> {
    // write config json
    let script = workspace.join("config.json");
    fs::write(&script, serde_json::to_vec(&param)?)?;

    let result = Command::new("cmd")
        .arg("/C")
        .arg(bin)
        .arg(script)
        .output()
        .unwrap();
    if !result.status.success() {
        let error_message = String::from_utf8(result.stderr).unwrap();
        Err(anyhow!("{}", error_message).into())
    } else {
        Ok(())
    }
}
