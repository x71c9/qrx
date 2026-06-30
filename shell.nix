{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = [
    pkgs.cargo
    pkgs.rustc
    pkgs.rustfmt
    pkgs.clippy
    pkgs.cargo-release
    # Add extra system dependencies here, e.g.:
    # pkgs.openssl
    # pkgs.pkg-config
  ];
  shellHook = ''
    export PATH="$PWD/scripts:$PATH"
    bash scripts/fetch-rebase.sh
    echo "cargo: $(cargo -V) | rustc: $(rustc -V)"
    echo "rustfmt: $(rustfmt --version) | clippy: $(cargo clippy --version)"
  '';
}
