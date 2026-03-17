# Minimal hardware config for x86_64 builds
# On a real device, replace this with the output of nixos-generate-config
#
# This provides the minimum required filesystem layout for NixOS to build.
# Adjust device paths (e.g. /dev/sda1, /dev/nvme0n1p1) to match your hardware.

{ config, lib, pkgs, ... }:

{
  # Root filesystem — REQUIRED by NixOS
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  # EFI system partition — required for GRUB EFI boot
  fileSystems."/boot" = {
    device = "/dev/disk/by-label/boot";
    fsType = "vfat";
    options = [ "fmask=0022" "dmask=0022" ];
  };

  # Swap (optional, recommended for devices with <8GB RAM)
  swapDevices = [
    { device = "/dev/disk/by-label/swap"; }
  ];

  # Hardware detection
  hardware.enableRedistributableFirmware = true;
}
