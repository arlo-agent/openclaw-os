{
  description = "OpenClaw OS — AI-native operating system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f {
        pkgs = import nixpkgs { inherit system; };
      });
    in {
      # NixOS system configurations
      nixosConfigurations.openclaw-x86 = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          ./modules/base.nix
          ./modules/openclaw.nix
          ./modules/voice.nix
          ./modules/shell.nix
        ];
      };

      nixosConfigurations.openclaw-pi = nixpkgs.lib.nixosSystem {
        system = "aarch64-linux";
        modules = [
          ./modules/base.nix
          ./modules/openclaw.nix
          ./modules/voice.nix
          ./modules/shell.nix
          ./modules/raspberry-pi.nix
        ];
      };

      # Dev shells for building the shell UI
      devShells = forAllSystems ({ pkgs }: {
        default = pkgs.mkShell {
          name = "openclaw-dev";

          buildInputs = with pkgs; [
            # Rust toolchain
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer

            # Build essentials
            pkg-config
            cmake

            # Graphics / Wayland
            wayland
            wayland-protocols
            libxkbcommon
            vulkan-loader
            vulkan-headers

            # X11 fallback (winit needs these)
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi

            # Font rendering
            fontconfig
            freetype
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            # Linux-specific
            libGL
            mesa
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
            # macOS frameworks
            AppKit
            CoreGraphics
            CoreServices
            Foundation
            Metal
            QuartzCore
          ]);

          # Ensure Vulkan/GL loaders are found at runtime
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
            vulkan-loader
            libxkbcommon
            wayland
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
            libGL
          ]);

          shellHook = ''
            echo "🦀 OpenClaw OS dev shell ready"
            echo "   cd shell && cargo run"
          '';
        };
      });
    };
}
