# OpenClaw OS — Branded boot experience
# GRUB2 with custom theme, logo, and OS branding
#
# Features:
# - Dark branded background with OpenClaw logo
# - Coral accent selection highlight
# - OS entries show "OpenClaw OS" instead of "NixOS"
# - Quiet boot with Plymouth splash after GRUB

{ config, pkgs, lib, ... }:

let
  # The GRUB theme directory — copied into the Nix store
  grubTheme = pkgs.runCommand "openclaw-grub-theme" {} ''
    mkdir -p $out
    cp ${../../assets/boot/background.png} $out/background.png
    cp ${../../assets/boot/theme.txt} $out/theme.txt
    cp ${../../assets/boot/select_c.png} $out/select_c.png
    cp ${../../assets/boot/select_w.png} $out/select_w.png
    cp ${../../assets/boot/select_e.png} $out/select_e.png
  '';

  # Plymouth theme — logo + coral orbital spinner
  plymouthTheme = pkgs.runCommand "openclaw-plymouth-theme" {} ''
    themeDir=$out/share/plymouth/themes/openclaw
    mkdir -p $themeDir

    # Copy and patch the .plymouth file with correct store paths
    sed \
      -e "s|@@IMAGEDIR@@|$themeDir|g" \
      -e "s|@@SCRIPTFILE@@|$themeDir/openclaw.script|g" \
      ${../../assets/plymouth/openclaw.plymouth} > $themeDir/openclaw.plymouth

    cp ${../../assets/plymouth/openclaw.script} $themeDir/
    cp ${../../assets/plymouth/logo.png} $themeDir/
    cp ${../../assets/plymouth/background.png} $themeDir/

    # Copy all spinner frames
    for f in ${../../assets/plymouth}/spinner-*.png; do
      cp "$f" $themeDir/
    done
  '';
in
{
  boot = {
    # Use GRUB2 instead of systemd-boot for full theming support
    loader = {
      systemd-boot.enable = lib.mkForce false;
      grub = {
        enable = true;
        device = "nodev";           # EFI install, not MBR
        efiSupport = true;
        efiInstallAsRemovable = false;
        useOSProber = false;        # We control our entries

        # Branding
        theme = grubTheme;
        splashImage = null;         # Theme handles the background
        backgroundColor = "#050810";

        # Timeout — show menu briefly then auto-boot
        timeoutStyle = "menu";

        # Custom menu entries — branded as "OpenClaw OS"
        extraEntries = ''
          # These are added alongside the auto-generated NixOS entries
        '';
      };
      efi.canTouchEfiVariables = true;

      # Rename NixOS generations in GRUB menu
      # NixOS generates entries like "NixOS - Configuration 42 (24.11...)"
      # We override the os-release to brand them as OpenClaw OS
    };

    # Quiet boot — no kernel messages, straight to Plymouth
    consoleLogLevel = 0;
    initrd.verbose = false;
    kernelParams = [
      "quiet"
      "splash"
      "udev.log_level=3"
      "vt.global_cursor_default=0"  # Hide text cursor
    ];

    # Plymouth splash screen (shows between GRUB and shell UI)
    plymouth = {
      enable = true;
      theme = "openclaw";
      themePackages = [ plymouthTheme ];
    };
  };

  # Brand the OS identity — this changes what shows in GRUB entries
  # and anywhere the OS name appears
  environment.etc."os-release".text = lib.mkForce ''
    NAME="OpenClaw OS"
    ID=openclaw
    ID_LIKE=nixos
    VERSION_ID="${config.system.nixos.release}"
    VERSION="${config.system.nixos.release} (${config.system.nixos.codeName})"
    PRETTY_NAME="OpenClaw OS ${config.system.nixos.release}"
    HOME_URL="https://openclaw.ai"
    SUPPORT_URL="https://github.com/arlo-agent/openclaw-os"
    BUG_REPORT_URL="https://github.com/arlo-agent/openclaw-os/issues"
    LOGO="openclaw"
  '';

  # Also override the NixOS branding in /etc/nixos
  environment.etc."nixos/label".text = "OpenClaw OS";
}
