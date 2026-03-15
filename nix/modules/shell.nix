# Shell UI module — the visual experience

{ config, pkgs, lib, ... }:

{
  # Minimal Wayland compositor (cage for now, custom later)
  # Cage runs a single application fullscreen — perfect for kiosk/shell
  programs.sway.enable = false; # We don't want sway
  
  # For Phase 1: cage compositor + Tauri shell
  # For Phase 2: custom smithay-based compositor + Iced shell
  
  systemd.services.openclaw-shell = {
    description = "OpenClaw Shell UI";
    wantedBy = [ "graphical.target" ];
    after = [ "openclaw-gateway.service" ];
    wants = [ "openclaw-gateway.service" ];

    serviceConfig = {
      Type = "simple";
      User = "openclaw";
      Group = "users";

      # Phase 1: cage compositor running the shell app
      ExecStart = "${pkgs.cage}/bin/cage -s -- /opt/openclaw-os/shell/openclaw-shell";

      Restart = "always";
      RestartSec = 3;

      # GPU access
      SupplementaryGroups = [ "video" "render" ];
    };

    environment = {
      WLR_LIBINPUT_NO_DEVICES = "1"; # Don't fail if no input devices
      XDG_RUNTIME_DIR = "/run/user/1000";
      # GPU rendering
      WLR_RENDERER = "vulkan"; # Prefer Vulkan, fall back to GLES
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
      noto-fonts-emoji   # Emoji support
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
  environment.systemPackages = with pkgs; [
    cage          # Wayland kiosk compositor
    wlr-randr    # Display configuration
  ];
}
