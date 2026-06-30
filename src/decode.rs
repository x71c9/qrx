use image::GrayImage;
use rxing::{
  BinaryBitmap, DecodeHints, Luma8LuminanceSource, MultiFormatReader, Reader,
  common::HybridBinarizer,
};

pub fn decode_qr(img: GrayImage) -> anyhow::Result<String> {
  let (w, h) = (img.width(), img.height());
  let pixels = img.into_raw();

  let source = Luma8LuminanceSource::new(pixels, w, h);
  let mut bitmap = BinaryBitmap::new(HybridBinarizer::new(source));

  let hints =
    DecodeHints::default().with(rxing::DecodeHintValue::TryHarder(true));

  let result = MultiFormatReader::default()
    .decode_with_hints(&mut bitmap, &hints)
    .map_err(|_| anyhow::anyhow!("no QR code detected"))?;

  Ok(result.getText().to_string())
}
