# Shell UI module — the visual experience
#
# Uses Sway compositor with greetd auto-login, replacing the
# previous Cage kiosk setup. Sway allows multiple windows
# (terminal, browser, shell) to coexist.

{ config, pkgs, lib, ... }:

let
  openclaw-shell = pkgs.callPackage ../packages/shell.nix {};

  # Sway configuration — loaded from assets
  swayConfig = pkgs.writeText "sway-config" (builtins.readFile ../../assets/sway/config);
in
{
  # Sway compositor
  programs.sway = {
    enable = true;
    wrapperFeatures.gtk = true;
    extraPackages = with pkgs; [
      swaylock
      swayidle
      wl-clipboard    # CRITICAL: enables clipboard copy/paste
      wlr-randr
      foot            # Terminal emulator
      chromium        # Web browser
    ];
  };

  # Greetd auto-login — replaces Cage's TTY1 session
  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        command = "${pkgs.sway}/bin/sway --config ${swayConfig}";
        user = "openclaw";
      };
    };
  };

  # Polkit is required for Sway seat management
  security.polkit.enable = true;

  # GPU drivers
  hardware.graphics = {
    enable = true;
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

  # Ensure the shell binary and display tools are available
  environment.systemPackages = [
    pkgs.wlr-randr     # Display configuration
    openclaw-shell      # The shell UI binary
  ];

  # Make sure openclaw-gateway starts before the greeter session
  systemd.services.greetd = {
    after = [ "openclaw-gateway.service" ];
    wants = [ "openclaw-gateway.service" ];
  };
}
