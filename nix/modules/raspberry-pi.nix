# Raspberry Pi 5 specific configuration

{ config, pkgs, lib, ... }:

{
  # Pi 5 boot
  boot.loader.systemd-boot.enable = lib.mkForce false;
  boot.loader.generic-extlinux-compatible.enable = true;

  # Pi 5 GPU
  hardware.raspberry-pi."5".fkms-3d.enable = true;

  # Pi-specific kernel
  boot.kernelPackages = pkgs.linuxPackages_rpi4; # TODO: rpi5 when available

  # GPIO access (for physical buttons, LEDs)
  users.users.openclaw.extraGroups = [ "gpio" "spi" "i2c" ];

  # Power management
  # Pi 5 has a power button — map it to screen toggle, not shutdown
  services.logind.extraConfig = ''
    HandlePowerKey=ignore
  '';

  # Recommended: official Pi 5 touchscreen or HDMI display
  # USB audio recommended (Pi audio jack is low quality)
}
