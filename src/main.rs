mod backend;
mod clipboard;
mod decode;

fn main() {
  if let Err(e) = run() {
    eprintln!("error: {e}");
    std::process::exit(1);
  }
}

const HELP: &str = "\
Selects a region of the screen, reads any QR code found in it, and copies
the result to the clipboard. An image file is decoded instead of the screen
when given.

Usage:
  qrx [-p] [-n] [file]

  qrx                crop a screen region, copy to clipboard
  qrx -p             crop a screen region, print and copy to clipboard
  qrx -pn            crop a screen region, print only
  qrx file.png       decode an image file, copy to clipboard
  qrx -pn file.png   decode an image file, print only

Options:
  -p, --print          print the decoded text to stdout
  -n, --no-clipboard   do not copy to the clipboard
  -v, --version        print the version
  -h, --help           show this help

-p and -n combine as -pn or -np. On Linux the text is copied to both the
clipboard and the primary selection.

Dependencies:
  X11       xclip
  Wayland   slurp, grim, wl-copy
  macOS     none; the terminal needs the Screen Recording permission
";

struct Args {
  print: bool,
  no_clipboard: bool,
  file: Option<std::path::PathBuf>,
}

fn parse_args() -> anyhow::Result<Args> {
  let mut print = false;
  let mut no_clipboard = false;
  let mut file = None;

  for arg in std::env::args().skip(1) {
    match arg.as_str() {
      "-p" | "--print" => print = true,
      "-n" | "--no-clipboard" => no_clipboard = true,
      "-pn" | "-np" => {
        print = true;
        no_clipboard = true;
      }
      "-v" | "--version" => {
        println!("{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
      }
      "-h" | "--help" => {
        print!("{HELP}");
        std::process::exit(0);
      }
      s if s.starts_with('-') => {
        anyhow::bail!("unknown flag: {s} (see qrx --help)")
      }
      _ => file = Some(std::path::PathBuf::from(&arg)),
    }
  }

  Ok(Args {
    print,
    no_clipboard,
    file,
  })
}

fn run() -> anyhow::Result<()> {
  let args = parse_args()?;

  let img = match args.file {
    Some(path) => image::open(&path)
      .map_err(|e| anyhow::anyhow!("failed to open {}: {e}", path.display()))?
      .into_luma8(),
    None => {
      let backend = backend::build()?;
      match backend.capture()? {
        Some(img) => img,
        None => std::process::exit(1),
      }
    }
  };

  let text = decode::decode_qr(img)?;

  if !args.no_clipboard {
    clipboard::copy_to_clipboard(&text)?;
  }

  if args.print {
    println!("{text}");
  }

  Ok(())
}
