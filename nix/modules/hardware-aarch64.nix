# Minimal hardware config for aarch64 builds (UTM/QEMU/ARM devices)
# On a real device, replace with output of nixos-generate-config
#
# Current UUIDs match Francis's UTM VM:
#   vda1 (vfat)  → boot  → 4A06-128F
#   vda2 (ext4)  → root  → 7f299365-7447-4fd3-8482-4789932eeb23

{ config, lib, pkgs, ... }:

{
  fileSystems."/" = {
    device = "/dev/disk/by-uuid/7f299365-7447-4fd3-8482-4789932eeb23";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-uuid/4A06-128F";
    fsType = "vfat";
    options = [ "fmask=0022" "dmask=0022" ];
  };

  # No swap partition on this VM
  swapDevices = [];

  hardware.enableRedistributableFirmware = true;
}
