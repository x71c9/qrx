use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {
  let mut child = Command::new("xclip")
    .args(["-selection", "clipboard"])
    .stdin(Stdio::piped())
    .spawn()?;

  child
    .stdin
    .as_mut()
    .expect("stdin piped")
    .write_all(text.as_bytes())?;

  let status = child.wait()?;
  if !status.success() {
    anyhow::bail!("xclip exited with status {status}");
  }
  Ok(())
}
