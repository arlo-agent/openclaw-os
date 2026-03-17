# OpenClaw Gateway service module

{ config, pkgs, lib, ... }:

{
  # OpenClaw gateway runs as a systemd service
  systemd.services.openclaw-gateway = {
    description = "OpenClaw AI Gateway";
    wantedBy = [ "multi-user.target" ];
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];

    serviceConfig = {
      Type = "simple";
      User = "openclaw";
      Group = "users";
      WorkingDirectory = "/home/openclaw/.openclaw/workspace";

      # Install openclaw globally if not present, then start
      ExecStartPre = pkgs.writeShellScript "openclaw-ensure" ''
        if ! command -v openclaw &> /dev/null; then
          ${pkgs.nodejs_22}/bin/npm install -g openclaw
        fi
      '';
      ExecStart = "${pkgs.nodejs_22}/bin/npx openclaw gateway start --foreground";

      Restart = "always";
      RestartSec = 5;

      # Security hardening
      NoNewPrivileges = true;
      ProtectSystem = "strict";
      ProtectHome = "read-only";
      ReadWritePaths = [
        "/home/openclaw/.openclaw"
        "/home/openclaw/.npm"
        "/tmp"
      ];
      PrivateTmp = true;
    };
  };

  # Workspace directory setup
  systemd.tmpfiles.rules = [
    "d /home/openclaw/.openclaw 0755 openclaw users -"
    "d /home/openclaw/.openclaw/workspace 0755 openclaw users -"
  ];
}
