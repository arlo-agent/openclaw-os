{
  description = "OpenClaw OS — AI-native operating system";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: {
    # x86_64 configuration (mini PCs, desktops)
    nixosConfigurations.openclaw-x86 = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./modules/base.nix
        ./modules/openclaw.nix
        ./modules/voice.nix
        ./modules/shell.nix
      ];
    };

    # aarch64 configuration (Raspberry Pi 5)
    nixosConfigurations.openclaw-pi = nixpkgs.lib.nixosSystem {
      system = "aarch64-linux";
      modules = [
        ./modules/base.nix
        ./modules/openclaw.nix
        ./modules/voice.nix
        ./modules/shell.nix
        ./modules/raspberry-pi.nix
      ];
    };
  };
}
