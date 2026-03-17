# Shell UI module — the visual experience
#
# Uses NixOS's built-in services.cage module which handles:
# - TTY allocation and getty conflict
# - PAM session (creates XDG_RUNTIME_DIR via pam_systemd)
# - Plymouth handoff
# - systemd.defaultUnit = graphical.target
# - Proper service dependencies and ordering

{ config, pkgs, lib, ... }:

let
  openclaw-shell = pkgs.callPackage ../packages/shell.nix {};
in
{
  # Use the built-in NixOS cage kiosk module
  services.cage = {
    enable = true;
    user = "openclaw";
    program = "${openclaw-shell}/bin/openclaw-shell";
    environment = {
      WLR_LIBINPUT_NO_DEVICES = "1";
    };
  };

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
    openclaw-shell     # The shell UI binary
  ];

  # Make sure openclaw-gateway starts before the shell
  systemd.services."cage-tty1" = {
    after = [ "openclaw-gateway.service" ];
    wants = [ "openclaw-gateway.service" ];
  };
}
