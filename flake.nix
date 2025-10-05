{
  description = "Vibe Kanban development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Rust toolchain from rust-toolchain.toml
        rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal.override {
          extensions = [ 
            "rust-src" 
            "rust-analyzer" 
            "rustfmt" 
            "rust-std"
            "cargo"
          ];
          targets = [ ];
        });

        # Node.js version (18+ required)
        nodejs = pkgs.nodejs_20;

        # Build dependencies for SQLx and other native deps
        nativeBuildDeps = with pkgs; [
          pkg-config
          openssl.dev
          sqlite.dev
          postgresql.lib
        ];

        # Runtime dependencies
        buildInputs = with pkgs; [
          openssl
          sqlite
          postgresql
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            
            # Node.js and package managers
            nodejs
            pnpm
            
            # Database tools
            sqlite
            postgresql
            
            # Development tools
            cargo-watch
            sqlx-cli
            
            # Build tools
            pkg-config
            openssl.dev
            clang
            llvm
            libclang
            
            # Other utilities
            concurrently
            git
            chromium
          ] ++ buildInputs ++ nativeBuildDeps;

          # Environment variables
          shellHook = ''
            echo "🦀 Rust toolchain: $(rustc --version)"
            echo "📦 Node.js: $(node --version)"
            echo "🧶 pnpm: $(pnpm --version)"
            echo ""
            echo "Available commands:"
            echo "  pnpm i              - Install dependencies"
            echo "  pnpm run dev        - Start development servers"
            echo "  cargo test          - Run Rust tests"
            echo "  npm run check       - Type check everything"
            echo ""
            
            # Ensure pnpm store directory exists
            export PNPM_HOME="$PWD/.pnpm"
            export PATH="$PNPM_HOME:$PATH"
            
            # Set up development environment
            export RUST_LOG=debug
            export RUST_BACKTRACE=1
            
            # SQLx offline mode (if needed)
            export SQLX_OFFLINE=true
          '';

          # Linker flags for openssl
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          
          # PKG_CONFIG_PATH for native dependencies
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.sqlite.dev}/lib/pkgconfig:${pkgs.postgresql.lib}/lib/pkgconfig";
          
          # Clang for bindgen
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          
          # Prevent cargo from trying to download rust toolchain
          RUSTUP_TOOLCHAIN = "stable";
        };

        # Optional: Provide packages that can be built
        packages = {
          # Could add build outputs here if needed
        };
      });
}
