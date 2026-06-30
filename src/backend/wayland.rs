use std::io::Read;
use std::process::{Command, Stdio};

use image::GrayImage;

pub struct WaylandBackend;

impl super::Backend for WaylandBackend {
  fn capture(&self) -> anyhow::Result<Option<GrayImage>> {
    let geom = slurp()?;
    let geom = match geom {
      Some(g) => g,
      None => return Ok(None),
    };

    let png = grim(&geom)?;
    let img = image::load_from_memory(&png)?.into_luma8();
    Ok(Some(img))
  }
}

fn slurp() -> anyhow::Result<Option<String>> {
  let output = Command::new("slurp").output()?;
  if !output.status.success() {
    return Ok(None);
  }
  let geom = String::from_utf8(output.stdout)?.trim().to_string();
  if geom.is_empty() {
    return Ok(None);
  }
  Ok(Some(geom))
}

fn grim(geom: &str) -> anyhow::Result<Vec<u8>> {
  let mut child = Command::new("grim")
    .args(["-g", geom, "-"])
    .stdout(Stdio::piped())
    .spawn()?;

  let mut png = Vec::new();
  child
    .stdout
    .as_mut()
    .expect("stdout piped")
    .read_to_end(&mut png)?;

  let status = child.wait()?;
  if !status.success() {
    anyhow::bail!("grim exited with status {status}");
  }
  Ok(png)
}
