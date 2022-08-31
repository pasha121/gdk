{
  description = "gdk-rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, ... }@inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        libsecp256k1 = import ./nix/libsecp256k1.nix { inherit pkgs; };

        libwally = import ./nix/libwally.nix { inherit pkgs libsecp256k1; };

        electrs = import ./nix/electrs.nix { inherit pkgs; };

        inherit (pkgs) lib stdenv;
      in
      {
        devShells.default = pkgs.mkShell
          {
            packages = with pkgs; [
              openssl
              pkg-config
              rust-bin.stable."1.56.0".default
            ] ++ lib.lists.optionals stdenv.hostPlatform.isDarwin [
              darwin.cctools
              darwin.apple_sdk.frameworks.Security
            ];

            WALLY_DIR = "${libwally}/lib";

            BITCOIND_EXEC = "${pkgs.bitcoin}/bin/bitcoind";

            ELECTRS_EXEC = "${electrs.bitcoin}/bin/electrs";

            ELECTRS_LIQUID_EXEC = "${electrs.liquid}/bin/electrs";

            ELEMENTSD_EXEC = "${pkgs.elementsd}/bin/elementsd";
          };
      }
    );
}
