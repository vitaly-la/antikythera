{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";
  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "antikythera";
        version = "0.1.0";

        nativeBuildInputs = with pkgs; [ rustfmt clippy makeWrapper ];

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
      };
    };
}
