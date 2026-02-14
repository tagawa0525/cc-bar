{
  description = "cc-bar: Claude Code Context Window Monitor for Cosmic DE";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      nixpkgsFor = forAllSystems (system: import nixpkgs {
        inherit system;
        overlays = [ self.overlays.default ];
      });
    in
    {
      # 他の flake から使えるオーバーレイ
      overlays.default = final: prev: {
        cc-bar = final.callPackage ./nix/package.nix { };
      };

      # パッケージ
      packages = forAllSystems (system: {
        cc-bar = nixpkgsFor.${system}.cc-bar;
        default = self.packages.${system}.cc-bar;
      });

      # 開発シェル（rust-overlay で最新ツールチェーンを使用）
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustToolchain
              pkg-config
              patchelf
              just
            ];

            buildInputs = with pkgs; [
              wayland
              wayland-protocols
              libxkbcommon
              libxkbcommon.dev
              fontconfig
              fontconfig.dev
              freetype
              freetype.dev
              expat
              expat.dev
              openssl
              openssl.dev
            ];

            shellHook = ''
              export PKG_CONFIG_PATH="${pkgs.lib.concatStringsSep ":" [
                "${pkgs.libxkbcommon.dev}/lib/pkgconfig"
                "${pkgs.fontconfig.dev}/lib/pkgconfig"
                "${pkgs.freetype.dev}/lib/pkgconfig"
                "${pkgs.expat.dev}/lib/pkgconfig"
                "${pkgs.openssl.dev}/lib/pkgconfig"
                "${pkgs.wayland.dev}/lib/pkgconfig"
                "${pkgs.wayland-protocols}/share/pkgconfig"
              ]}''${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
                pkgs.wayland
                pkgs.libxkbcommon
                pkgs.fontconfig
                pkgs.freetype
                pkgs.expat
              ]}:$LD_LIBRARY_PATH"
            '';
          };
        });
    };
}
