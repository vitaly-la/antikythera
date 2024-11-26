{ pkgs ? import <nixpkgs> { } }:
pkgs.rustPlatform.buildRustPackage {
  pname = "antikythera";
  version = "0.1.0";

  nativeBuildInputs = with pkgs; [ makeWrapper ];

  buildInputs = with pkgs; [
    SDL2
    SDL2_gfx
    SDL2_image
    SDL2_ttf
  ];

  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;

  postInstall = ''
    cp -r resources $out
    wrapProgram $out/bin/antikythera --set RESOURCES_DIR $out/resources
  '';
}
