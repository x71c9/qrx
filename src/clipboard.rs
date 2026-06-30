use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {
  #[cfg(target_os = "macos")]
  let args: &[&str] = &[];
  #[cfg(target_os = "macos")]
  let program = "pbcopy";

  #[cfg(not(target_os = "macos"))]
  if std::env::var("WAYLAND_DISPLAY").is_ok() {
    // wl-copy --primary for primary selection, then again for clipboard
    for extra in &[Some("--primary"), None] {
      let mut cmd = Command::new("wl-copy");
      if let Some(flag) = extra {
        cmd.arg(flag);
      }
      let mut child = cmd.stdin(Stdio::piped()).spawn()?;
      child
        .stdin
        .as_mut()
        .expect("stdin piped")
        .write_all(text.as_bytes())?;
      let status = child.wait()?;
      if !status.success() {
        anyhow::bail!("wl-copy exited with status {status}");
      }
    }
    return Ok(());
  }

  #[cfg(not(target_os = "macos"))]
  let (program, args): (&str, &[&str]) =
    ("xclip", &["-selection", "clipboard"]);

  #[cfg(not(target_os = "macos"))]
  // also copy to primary selection for middle-click paste
  {
    let mut child = Command::new("xclip")
      .args(["-selection", "primary"])
      .stdin(Stdio::piped())
      .spawn()?;
    child
      .stdin
      .as_mut()
      .expect("stdin piped")
      .write_all(text.as_bytes())?;
    child.wait()?;
  }

  let mut child = Command::new(program)
    .args(args)
    .stdin(Stdio::piped())
    .spawn()?;

  child
    .stdin
    .as_mut()
    .expect("stdin piped")
    .write_all(text.as_bytes())?;

  let status = child.wait()?;
  if !status.success() {
    anyhow::bail!("{program} exited with status {status}");
  }
  Ok(())
}
