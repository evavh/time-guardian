{
  description = "Screen time control tool";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
  };

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
    in
    {
      packages.${system}.default = nixpkgs.legacyPackages.${system}.callPackage ./default.nix { };
      nixosModules.default = import ./nix_module.nix;
    };
}
