# Nix derivation for the OpenClaw Shell UI (Rust/Iced)
#
# Builds the shell binary from source and installs it to /bin/openclaw-shell.
# The NixOS module (shell.nix) references this package.

{ pkgs, lib, ... }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "openclaw-shell";
  version = "0.1.0";

  src = ../../shell;

  cargoLock = {
    lockFile = ../../shell/Cargo.lock;
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
    cmake
  ];

  buildInputs = with pkgs; [
    # Wayland
    wayland
    wayland-protocols
    libxkbcommon

    # Vulkan/GL
    vulkan-loader
    vulkan-headers
    libGL
    mesa

    # X11 fallback (winit/iced needs these)
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi

    # Font rendering
    fontconfig
    freetype
  ];

  # Runtime library paths for GPU drivers
  postFixup = ''
    patchelf --add-rpath ${lib.makeLibraryPath (with pkgs; [
      vulkan-loader
      libxkbcommon
      wayland
      libGL
    ])} $out/bin/openclaw-shell
  '';

  meta = with lib; {
    description = "OpenClaw OS Shell — ambient display, conversation, and agent UI";
    homepage = "https://github.com/arlo-agent/openclaw-os";
    license = licenses.mit;
    platforms = platforms.linux;
    mainProgram = "openclaw-shell";
  };
}
