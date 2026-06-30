mod backend;
mod clipboard;
mod decode;

fn main() {
  if let Err(e) = run() {
    eprintln!("error: {e}");
    std::process::exit(1);
  }
}

fn run() -> anyhow::Result<()> {
  let backend = backend::build()?;

  let img = match backend.capture()? {
    Some(img) => img,
    None => std::process::exit(1),
  };

  let text = decode::decode_qr(img)?;
  clipboard::copy_to_clipboard(&text)?;
  println!("{text}");

  Ok(())
}
