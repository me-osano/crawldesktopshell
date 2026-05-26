{
  description = "CrawlDS - a Wayland desktop shell built with Quickshell + Rust";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    quickshell = {
      url = "git+https://git.outfoxxed.me/outfoxxed/quickshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      quickshell,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "clippy"
            "rustfmt"
            "rust-analyzer"
          ];
        };
        qsPkg = quickshell.packages.${system}.default;
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config
            cmake
            qsPkg
          ];

          buildInputs = with pkgs; [
            openssl
            openssl.dev
            dbus
            dbus.dev
            udev
            libgit2
            libsoup_3
            # QML tooling
            qt6.qtdeclarative
            qt6.qttools
          ];

          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
            pkgs.openssl.dev
            pkgs.dbus.dev
            pkgs.udev
            pkgs.libgit2
            pkgs.libsoup_3
          ];

          RUST_BACKTRACE = 1;
          RUST_LOG = "debug";

          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";

          shellHook = ''
            echo "🦀 Rust $(rustc --version)"
            echo "📦 Cargo $(cargo --version)"
            echo "🖥️  Quickshell ${qsPkg.version or "?"}"
          '';
        };
      }
    );
}
