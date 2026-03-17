# Shell UI module — the visual experience

{ config, pkgs, lib, ... }:

let
  openclaw-shell = pkgs.callPackage ../packages/shell.nix {};
in
{
  # Minimal Wayland compositor (cage for now, custom later)
  # Cage runs a single application fullscreen — perfect for kiosk/shell
  programs.sway.enable = false; # We don't want sway
  
  # For Phase 1: cage compositor + Tauri shell
  # For Phase 2: custom smithay-based compositor + Iced shell
  
  # Shell service — only starts if the binary exists
  # For development: build the shell locally then symlink to /opt/openclaw-os/shell/
  # For production: this will use a Nix-built package
  systemd.services.openclaw-shell = {
    description = "OpenClaw Shell UI";
    wantedBy = [ "graphical.target" ];
    after = [ "openclaw-gateway.service" "network.target" ];
    wants = [ "openclaw-gateway.service" ];

    serviceConfig = {
      Type = "simple";
      User = "openclaw";
      Group = "users";

      # cage compositor running the Nix-built shell binary fullscreen
      ExecStart = "${pkgs.cage}/bin/cage -s -- ${openclaw-shell}/bin/openclaw-shell";

      Restart = "on-failure";
      RestartSec = 5;

      # GPU access
      SupplementaryGroups = [ "video" "render" ];
    };

    environment = {
      WLR_LIBINPUT_NO_DEVICES = "1";
      XDG_RUNTIME_DIR = "/run/user/1000";
      WLR_RENDERER = "vulkan";
    };
  };

  # GPU drivers
  hardware.graphics = {
    enable = true;
    # Enable Vulkan
  };

  # Fonts — the typography matters
  fonts = {
    enableDefaultPackages = false;
    packages = with pkgs; [
      inter              # Primary UI font
      jetbrains-mono     # Code/mono font
      noto-fonts-color-emoji   # Emoji support
    ];
    fontconfig = {
      defaultFonts = {
        sansSerif = [ "Inter" ];
        monospace = [ "JetBrains Mono" ];
        emoji = [ "Noto Color Emoji" ];
      };
      # Subpixel rendering for crisp text
      subpixel.rgba = "rgb";
      hinting.style = "slight";
      antialias = true;
    };
  };

  # Cursor theme (hidden most of the time, but needed for touch fallback)
  environment.systemPackages = [
    pkgs.cage          # Wayland kiosk compositor
    pkgs.wlr-randr     # Display configuration
    openclaw-shell     # The shell UI binary
  ];
}
