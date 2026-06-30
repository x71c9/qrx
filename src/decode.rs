use image::GrayImage;
use rqrr::PreparedImage;

pub fn decode_qr(img: GrayImage) -> anyhow::Result<String> {
  let mut prepared = PreparedImage::prepare(img);
  let grids = prepared.detect_grids();
  if grids.is_empty() {
    anyhow::bail!("no QR code detected");
  }
  let (_meta, content) = grids[0].decode()?;
  Ok(content)
}
