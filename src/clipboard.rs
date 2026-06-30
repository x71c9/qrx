use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {
  #[cfg(target_os = "macos")]
  let args: &[&str] = &[];
  #[cfg(target_os = "macos")]
  let program = "pbcopy";

  #[cfg(not(target_os = "macos"))]
  let (program, args): (&str, &[&str]) =
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
      ("wl-copy", &[])
    } else {
      ("xclip", &["-selection", "clipboard"])
    };

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
