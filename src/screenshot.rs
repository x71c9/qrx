use image::GrayImage;
use x11rb::{connection::Connection, protocol::xproto::ConnectionExt};

use crate::capture::Region;

pub fn capture(
  conn: &x11rb::rust_connection::RustConnection,
  screen_num: usize,
  region: &Region,
) -> anyhow::Result<GrayImage> {
  let root = conn.setup().roots[screen_num].root;

  let reply = conn.get_image(
    x11rb::protocol::xproto::ImageFormat::Z_PIXMAP,
    root,
    region.x,
    region.y,
    region.width,
    region.height,
    !0,
  )?;
  let reply = reply.reply()?;

  // X11 ZPixmap is BGRA (or BGRx) 32-bit per pixel
  let data = reply.data;
  let w = region.width as u32;
  let h = region.height as u32;
  let mut gray = GrayImage::new(w, h);

  for (i, pixel) in gray.pixels_mut().enumerate() {
    let base = i * 4;
    let b = data[base] as u32;
    let g = data[base + 1] as u32;
    let r = data[base + 2] as u32;
    // Rec.601 luma
    let luma = (r * 299 + g * 587 + b * 114) / 1000;
    *pixel = image::Luma([luma as u8]);
  }

  Ok(gray)
}
