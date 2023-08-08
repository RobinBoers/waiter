{ nixpkgs ? import <nixpkgs> { }}:

let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"; 
  pinnedPkgs = builtins.fetchTarball "https://github.com/nixos/nixpkgs/archive/nixpkgs-unstable.tar.gz";
  pkgs = import pinnedPkgs {
    overlays = [ (import rustOverlay) ];
  };
in pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust impl
    rust-bin.stable.latest.default
    rust-analyzer
  ];

  RUST_BACKTRACE = 1;
}
