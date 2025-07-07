{
  description = "SwarmReport - Real-time system monitoring for distributed environments";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Common build inputs
        buildInputs = with pkgs; [
          pkg-config
          protobuf
          libiconv
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          protobuf
          rustToolchain
        ];

        # Build the full workspace
        swarmreport = pkgs.rustPlatform.buildRustPackage {
          pname = "swarmreport";
          version = "0.1.0";
          
          src = pkgs.lib.cleanSource ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;
          
          # Set environment variables for build
          PKG_CONFIG_PATH = "${pkgs.protobuf}/lib/pkgconfig";
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";

          meta = with pkgs.lib; {
            description = "Real-time system monitoring for distributed environments";
            homepage = "https://github.com/yourusername/swarmreport";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        # Individual binary packages
        sentinel = pkgs.rustPlatform.buildRustPackage {
          pname = "swarmreport-sentinel";
          version = "0.1.0";
          
          src = pkgs.lib.cleanSource ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;
          
          # Build only the sentinel binary
          cargoBuildFlags = [ "--bin" "sentinel" ];
          cargoTestFlags = [ "--bin" "sentinel" ];

          # Set environment variables for build
          PKG_CONFIG_PATH = "${pkgs.protobuf}/lib/pkgconfig";
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";

          meta = with pkgs.lib; {
            description = "SwarmReport sentinel server";
            homepage = "https://github.com/yourusername/swarmreport";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

        reporter = pkgs.rustPlatform.buildRustPackage {
          pname = "swarmreport-reporter";
          version = "0.1.0";
          
          src = pkgs.lib.cleanSource ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;
          
          # Build only the reporter binary
          cargoBuildFlags = [ "--bin" "reporter" ];
          cargoTestFlags = [ "--bin" "reporter" ];

          # Set environment variables for build
          PKG_CONFIG_PATH = "${pkgs.protobuf}/lib/pkgconfig";
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${pkgs.protobuf}/include";

          meta = with pkgs.lib; {
            description = "SwarmReport client reporter";
            homepage = "https://github.com/yourusername/swarmreport";
            license = licenses.mit;
            maintainers = [ ];
          };
        };

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = buildInputs ++ [
            rustToolchain
            pkgs.cargo-audit
            pkgs.cargo-watch
            pkgs.cargo-edit
          ];

          shellHook = ''
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            export PKG_CONFIG_PATH="${pkgs.protobuf}/lib/pkgconfig:$PKG_CONFIG_PATH"
            export PROTOC="${pkgs.protobuf}/bin/protoc"
            export PROTOC_INCLUDE="${pkgs.protobuf}/include"
            echo "🚀 SwarmReport development environment loaded!"
            echo "Available commands:"
            echo "  cargo build          - Build the project"
            echo "  cargo test           - Run tests"
            echo "  cargo run --bin sentinel  - Run sentinel server"
            echo "  cargo run --bin reporter  - Run reporter client"
            echo "  cargo audit          - Security audit"
            echo "  cargo watch -x check - Watch for changes"
          '';
        };

        # Packages
        packages = {
          inherit sentinel reporter swarmreport;
          default = swarmreport;
        };

        # Apps for easy running
        apps = {
          sentinel = flake-utils.lib.mkApp {
            drv = sentinel;
            name = "sentinel";
          };
          reporter = flake-utils.lib.mkApp {
            drv = reporter;
            name = "reporter";
          };
          default = flake-utils.lib.mkApp {
            drv = swarmreport;
            name = "sentinel";
          };
        };

        # Simple checks for CI
        checks = {
          inherit swarmreport sentinel reporter;
        };
      });
}
