#[cfg(not(target_os = "macos"))]
pub mod x11;

#[cfg(not(target_os = "macos"))]
pub mod wayland;

#[cfg(target_os = "macos")]
pub mod macos;

use image::GrayImage;

pub trait Backend {
  fn capture(&self) -> anyhow::Result<Option<GrayImage>>;
}

pub enum DisplayServer {
  #[cfg(not(target_os = "macos"))]
  Wayland,
  #[cfg(not(target_os = "macos"))]
  X11,
  #[cfg(target_os = "macos")]
  MacOs,
}

pub fn detect() -> anyhow::Result<DisplayServer> {
  #[cfg(target_os = "macos")]
  return Ok(DisplayServer::MacOs);

  #[cfg(not(target_os = "macos"))]
  {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
      return Ok(DisplayServer::Wayland);
    }
    if std::env::var("DISPLAY").is_ok() {
      return Ok(DisplayServer::X11);
    }
    anyhow::bail!(
      "no display found: neither WAYLAND_DISPLAY nor DISPLAY is set"
    )
  }
}

pub fn build() -> anyhow::Result<Box<dyn Backend>> {
  match detect()? {
    #[cfg(not(target_os = "macos"))]
    DisplayServer::Wayland => Ok(Box::new(wayland::WaylandBackend)),
    #[cfg(not(target_os = "macos"))]
    DisplayServer::X11 => Ok(Box::new(x11::X11Backend::new()?)),
    #[cfg(target_os = "macos")]
    DisplayServer::MacOs => Ok(Box::new(macos::MacOsBackend)),
  }
}
