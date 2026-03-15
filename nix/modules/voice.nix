# Voice pipeline module — always-on wake word + STT/TTS

{ config, pkgs, lib, ... }:

{
  # Voice pipeline service
  # This runs the wake word detector and manages the STT/TTS flow
  systemd.services.openclaw-voice = {
    description = "OpenClaw Voice Pipeline";
    wantedBy = [ "multi-user.target" ];
    after = [ "pipewire.service" "openclaw-gateway.service" ];
    wants = [ "pipewire.service" ];

    serviceConfig = {
      Type = "simple";
      User = "openclaw";
      Group = "users";
      WorkingDirectory = "/home/openclaw/.openclaw";

      # Voice pipeline binary (to be built)
      ExecStart = "/opt/openclaw-os/voice/openclaw-voice";

      Restart = "always";
      RestartSec = 3;

      # Needs audio access
      SupplementaryGroups = [ "audio" ];
    };

    # Environment variables for voice config
    environment = {
      OPENCLAW_VOICE_WAKE_WORD = "hey claw";
      OPENCLAW_VOICE_STT_PROVIDER = "whisper"; # whisper | deepgram | local
      OPENCLAW_VOICE_TTS_PROVIDER = "elevenlabs"; # elevenlabs | openai | piper
      OPENCLAW_VOICE_VAD_THRESHOLD = "0.5";
      OPENCLAW_VOICE_SILENCE_TIMEOUT_MS = "2000";
      PIPEWIRE_RUNTIME_DIR = "/run/user/1000";
    };
  };

  # PipeWire echo cancellation module
  services.pipewire.extraConfig.pipewire."20-echo-cancel" = {
    "context.modules" = [
      {
        name = "libpipewire-module-echo-cancel";
        args = {
          "audio.cancel-method" = "webrtc";
          "source.props" = {
            "node.name" = "echo-cancel-source";
            "node.description" = "Echo-Cancelled Microphone";
          };
          "sink.props" = {
            "node.name" = "echo-cancel-sink";
            "node.description" = "Echo-Cancel Speaker";
          };
        };
      }
    ];
  };

  # Required packages for voice
  environment.systemPackages = with pkgs; [
    # Local STT fallback
    # whisper-cpp

    # Local TTS fallback  
    # piper-tts

    # Audio utilities
    sox
    ffmpeg
  ];
}
