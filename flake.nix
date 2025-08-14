{
  description = "piss fan flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            twitch-cli

            npm
            bun
            nodejs

            typescript
            redis
            postgresql

            git
            jq
            ripgrep
          ];

          shellHook = ''
            bun --version
          '';
        };
      }
    );
}
