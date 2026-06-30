use std::io::Read;
use std::process::{Command, Stdio};

use image::GrayImage;

pub struct MacOsBackend;

impl super::Backend for MacOsBackend {
  fn capture(&self) -> anyhow::Result<Option<GrayImage>> {
    let mut child = Command::new("screencapture")
      .args(["-i", "-s", "-"])
      .stdout(Stdio::piped())
      .spawn()?;

    let mut png = Vec::new();
    child
      .stdout
      .as_mut()
      .expect("stdout piped")
      .read_to_end(&mut png)?;

    let status = child.wait()?;
    if !status.success() || png.is_empty() {
      return Ok(None);
    }

    let img = image::load_from_memory(&png)?.into_luma8();
    Ok(Some(img))
  }
}
