{
  description = "Real-time library for SQLx";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      # Supported systems
      systems = [
        "aarch64-linux"
        "i686-linux"
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShell = forAllSystems (
        system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          makePkgConfigPath = pkgs.lib.makeSearchPathOutput "out" "lib/pkgconfig";
        in
        pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            at-spi2-atk
            atkmm
            cairo
            gdk-pixbuf
            glib
            gobject-introspection
            gobject-introspection.dev
            gtk3
            harfbuzz
            librsvg
            libsoup_3
            pango
            webkitgtk_4_1
            webkitgtk_4_1.dev

            # Additional libraries not mentionned
            openssl
            libsysprof-capture
            libthai
            libdatrie
            libselinux
            lerc
            libsepol
            xorg.libXdmcp
            util-linux.dev
            pcre2
            sqlite
            libpsl
            libxkbcommon
            libepoxy
            xorg.libXtst
            nghttp2
          ];

          # Required for pkg-config to find the libraries
          packages = with pkgs; [
            pkgconf

            rust-bin.stable.latest.default
          ];

          # https://github.com/tauri-apps/tauri/issues/8929
          NO_STRIP = "true";

          PKG_CONFIG_PATH =
            with pkgs;
            makePkgConfigPath [
              glib.dev
              libsoup_3.dev
              webkitgtk_4_1.dev
              at-spi2-atk.dev
              gtk3.dev
              gdk-pixbuf.dev
              cairo.dev
              pango.dev
              harfbuzz.dev
            ];
        }
      );
    };
}
