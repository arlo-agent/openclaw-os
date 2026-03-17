# Base NixOS configuration for OpenClaw OS
# This is the foundation — minimal, secure, fast boot

{ config, pkgs, lib, ... }:

{
  # System identity
  networking.hostName = "openclaw";
  time.timeZone = "UTC"; # Agent handles user timezone

  # Boot config is in boot.nix (branded GRUB + Plymouth)

  # Pull graphical.target into default boot so openclaw-shell.service starts
  systemd.targets.graphical.wantedBy = [ "default.target" ];

  # Networking
  networking.networkmanager.enable = true;

  # Audio — PipeWire (low latency, modern)
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    # Low latency config for voice
    extraConfig.pipewire."10-low-latency" = {
      "context.properties" = {
        "default.clock.rate" = 48000;
        "default.clock.quantum" = 256;
        "default.clock.min-quantum" = 128;
      };
    };
  };

  # Bluetooth
  hardware.bluetooth = {
    enable = true;
    powerOnBoot = true;
    settings.General.Experimental = true;
  };

  # Users — single unprivileged user for OpenClaw
  users.users.openclaw = {
    isNormalUser = true;
    home = "/home/openclaw";
    extraGroups = [ "audio" "video" "networkmanager" "bluetooth" ];
    # No password — device boots directly into agent
  };

  # Admin user for maintenance/debugging (TTY + SSH)
  users.users.admin = {
    isNormalUser = true;
    extraGroups = [ "wheel" "networkmanager" "video" ];
    initialPassword = "openclaw";  # Change on first login!
  };

  # Allow admin to sudo
  security.sudo.wheelNeedsPassword = true;

  # SSH for remote debugging
  services.openssh = {
    enable = true;
    settings = {
      PasswordAuthentication = true;  # For initial setup; switch to keys later
      PermitRootLogin = "no";
    };
  };

  # Auto-login to openclaw user
  services.getty.autologinUser = "openclaw";


  # Essential packages
  environment.systemPackages = with pkgs; [
    # Core
    nodejs_22
    git
    curl
    wget
    htop

    # Audio tools
    alsa-utils
    pavucontrol

    # System
    usbutils
    pciutils
  ];

  # Firewall — locked down
  networking.firewall = {
    enable = true;
    allowedTCPPorts = [ 22 ]; # SSH for debugging
  };

  # Automatic garbage collection
  nix = {
    gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 14d";
    };
    settings = {
      auto-optimise-store = true;
      experimental-features = [ "nix-command" "flakes" ];
    };
  };

  # NixOS version
  system.stateVersion = "24.11";
}
