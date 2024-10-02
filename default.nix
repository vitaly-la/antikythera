{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "antikythera";
  version = "0.1.0";

  buildInputs = with pkgs; [
    SDL2
    SDL2_gfx
    SDL2_image
    SDL2_ttf
  ];

  src = pkgs.lib.cleanSource ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
