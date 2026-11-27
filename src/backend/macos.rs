use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use image::GrayImage;

pub struct MacOsBackend;

// screencapture writes only to files (`-` would be a file named `-`):
// the selection goes to a file in $TMPDIR, private to the user on macOS
fn capture_path() -> PathBuf {
  let nanos = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_nanos())
    .unwrap_or(0);
  std::env::temp_dir().join(format!("qrx-{}-{nanos}.png", std::process::id()))
}

impl super::Backend for MacOsBackend {
  fn capture(&self) -> anyhow::Result<Option<GrayImage>> {
    let path = capture_path();
    let output = Command::new("screencapture")
      .args(["-i", "-s"])
      .arg(&path)
      .output();
    let png = fs::read(&path);
    let _ = fs::remove_file(&path);
    let output = output?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    match png {
      Ok(png) if !png.is_empty() => {
        Ok(Some(image::load_from_memory(&png)?.into_luma8()))
      }
      // No image and no message: the selection was cancelled (Esc)
      _ if stderr.trim().is_empty() => Ok(None),
      // "could not create image from rect" when the permission is missing
      _ => anyhow::bail!(
        "screen capture failed: {}. The terminal running qrx needs the \
         Screen Recording permission: System Settings > Privacy & Security \
         > Screen & System Audio Recording, then restart the terminal",
        stderr.trim()
      ),
    }
  }
}
