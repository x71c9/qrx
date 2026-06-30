mod capture;
mod clipboard;
mod decode;
mod screenshot;

use x11rb::rust_connection::RustConnection;

fn main() {
  if let Err(e) = run() {
    eprintln!("error: {e}");
    std::process::exit(1);
  }
}

fn run() -> anyhow::Result<()> {
  let (conn, screen_num) = RustConnection::connect(None)?;

  let region = match capture::select_region(&conn, screen_num)? {
    Some(r) => r,
    None => {
      eprintln!("selection cancelled");
      return Ok(());
    }
  };

  let img = screenshot::capture(&conn, screen_num, &region)?;
  let text = decode::decode_qr(img)?;

  clipboard::copy_to_clipboard(&text)?;
  println!("{text}");

  Ok(())
}
