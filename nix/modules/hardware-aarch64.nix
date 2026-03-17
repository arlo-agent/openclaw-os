# Minimal hardware config for aarch64 builds (UTM/QEMU/ARM devices)
# On a real device, replace with output of nixos-generate-config

{ config, lib, pkgs, ... }:

{
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-label/boot";
    fsType = "vfat";
    options = [ "fmask=0022" "dmask=0022" ];
  };

  swapDevices = [
    { device = "/dev/disk/by-label/swap"; }
  ];

  hardware.enableRedistributableFirmware = true;
}
