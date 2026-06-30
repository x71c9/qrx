# PKGBIN

One-line description of what this tool does.

## Installation

### NixOS / Nix — NUR

Add the NUR channel if you haven't already:

```bash
nix-channel --add https://github.com/nix-community/NUR/archive/master.tar.gz nur
nix-channel --update
```

Then install:

```nix
# configuration.nix
environment.systemPackages = [
  nur.repos.x71c9.PKGBIN
];
```

Or ad-hoc:

```bash
nix-env -f '<nixpkgs>' -iA nur.repos.x71c9.PKGBIN
```

### Linux — AUR (Arch Linux)

```bash
yay -S PKGBIN-bin
# or
paru -S PKGBIN-bin
```

### crates.io — Cargo

```bash
cargo install PKGCRATE
```

### macOS — Homebrew

```bash
brew install x71c9/x71c9/PKGBIN
```

### Pre-built binaries

Download the latest release for your platform from the
[releases page](https://github.com/x71c9/PKGBIN/releases).

| Platform | File |
|----------|------|
| Linux x86\_64 | `PKGBIN-x86_64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `PKGBIN-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `PKGBIN-x86_64-apple-darwin.tar.gz` |

## Usage

```bash
PKGBIN --help
```

## License

MIT
