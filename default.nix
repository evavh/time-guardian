{
  pkgs ? import <nixpkgs> { },
}:
let
  src = pkgs.lib.cleanSource ./.;
  cargoTOML = pkgs.lib.importTOML "${src}/Cargo.toml";
in
pkgs.rustPlatform.buildRustPackage {
  pname = cargoTOML.package.name;
  version = cargoTOML.package.version;
  cargoLock = {
    lockFile = "${src}/Cargo.lock";
    outputHashes = {
      "break-enforcer-0.3.2" = "sha256-w9Ov5+JODhZcb5n/FU4qdnAyoaSmY0mAkbDFJiU3EBg=";
    };
  };
  inherit src;
}
