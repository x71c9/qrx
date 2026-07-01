mod backend;
mod clipboard;
mod decode;

fn main() {
  if let Err(e) = run() {
    eprintln!("error: {e}");
    std::process::exit(1);
  }
}

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
      s if s.starts_with('-') => anyhow::bail!("unknown flag: {s}"),
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
