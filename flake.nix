{
  description = "Node.js development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustup
            gcc
            gnumake
            lld
            pkg-config
            openssl
          ];

          shellHook = ''
            rustup toolchain install stable

            export PATH="$HOME/.cargo/bin:$PATH"

            echo "Rust:  $(rustc --version)"
            echo "Cargo: $(cargo --version)"
          '';
        };
      });
}
