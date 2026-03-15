# Architecture

## Overview

OpenClaw OS is built in layers, each replaceable and independently testable.

```
┌─────────────────────────────────────────────────┐
│                  SHELL UI                        │
│        Custom compositor + Iced/Rust UI          │
│     Ambient display · Cards · Visualizations     │
├─────────────────────────────────────────────────┤
│                VOICE PIPELINE                    │
│   Wake word → VAD → STT → Agent → TTS → Audio   │
├─────────────────────────────────────────────────┤
│              OPENCLAW GATEWAY                    │
│     Agent runtime · Tools · Integrations         │
├─────────────────────────────────────────────────┤
│                  NIXOS BASE                      │
│    Kernel · Drivers · Networking · Services      │
└─────────────────────────────────────────────────┘
```

## Layer 1: NixOS Base

The foundation. Handles everything below the application layer.

### Why NixOS?

- **Declarative:** The entire system is defined in `configuration.nix`. No snowflake configs.
- **Atomic updates:** System updates either fully succeed or fully roll back. No half-broken states.
- **Reproducible:** Given the same config, every machine is identical. Critical for shipping a product.
- **Generations:** Every config change creates a new "generation." Boot menu lets you roll back to any previous state.
- **Minimal base:** Strip out everything we don't need. No GUI package managers, no office suites, no games.

### Base System Includes

- Linux kernel (latest stable, with RT patches for audio latency)
- PipeWire (audio — replaces PulseAudio/JACK, lower latency)
- NetworkManager (WiFi, Ethernet, auto-connect)
- Bluetooth stack
- Node.js 22 LTS (for OpenClaw)
- Minimal getty for emergency TTY access
- SSH server (optional, for remote management)

### What's NOT Included

- No traditional desktop environment (GNOME, KDE, etc.)
- No display manager (GDM, SDDM, etc.)
- No traditional window manager
- No package manager UI
- No browser (unless needed for OpenClaw connector)

## Layer 2: OpenClaw Gateway

Standard OpenClaw installation, running as a systemd service.

```nix
# NixOS module for OpenClaw
systemd.services.openclaw-gateway = {
  description = "OpenClaw AI Gateway";
  wantedBy = [ "multi-user.target" ];
  after = [ "network-online.target" ];
  serviceConfig = {
    ExecStart = "${pkgs.openclaw}/bin/openclaw gateway start --foreground";
    Restart = "always";
    RestartSec = 5;
    User = "openclaw";
    WorkingDirectory = "/home/openclaw/.openclaw/workspace";
  };
};
```

The gateway starts on boot, connects to configured messaging channels, and runs the agent. This is the brain.

### Local Integrations

Because we control the OS, OpenClaw gets deeper access:

- **System audio:** Direct PipeWire integration for voice I/O
- **Display control:** Can turn screen on/off, adjust brightness
- **Bluetooth:** Pair speakers, headphones, keyboards
- **Network:** Connect to WiFi, check connectivity
- **System updates:** `nixos-rebuild switch` through the agent
- **Hardware sensors:** Temperature, CPU, memory, disk

## Layer 3: Voice Pipeline

Always-on voice interaction. This is what makes it feel alive.

See [voice.md](voice.md) for the full voice pipeline design.

### Summary

```
Microphone → Wake Word Detection (local, always-on)
                    ↓ triggered
              VAD (Voice Activity Detection)
                    ↓ speech segment
              STT (Speech-to-Text)
                    ↓ transcript
              OpenClaw Agent (process turn)
                    ↓ response text
              TTS (Text-to-Speech)
                    ↓ audio
              Speaker output
```

Key requirement: wake word detection runs locally with <1% CPU. No cloud dependency for activation.

## Layer 4: Shell UI

The visual layer. This is where the Apple-level polish lives.

See [ui-design.md](ui-design.md) for the full design vision.

### Tech Stack Decision: Rust + Iced

Why not Electron/web:
- We're building an OS shell, not a web app. Electron means shipping Chromium. Heavy, slow boot, feels like a website.
- Can't do custom compositor-level effects (blur, transparency, smooth transitions between states)
- Memory overhead matters on Pi/low-end hardware

Why not GTK/Qt:
- GTK4 is good but GNOME-centric. Customizing deeply means fighting the toolkit
- Qt is powerful but complex, licensing considerations
- Neither gives us the level of animation control we want

Why Rust + Iced:
- **Iced** is a Rust GUI framework inspired by Elm. GPU-accelerated, 60fps, cross-platform
- Runs directly on Wayland via iced_sctk (Smithay Client Toolkit) — no X11 needed
- Full control over every pixel. Custom shaders, animations, transitions
- Memory-safe, fast, small binary
- Can embed as a Wayland client inside a minimal compositor, or BE the compositor
- Community is active, framework is maturing fast

Alternative worth watching: **COSMIC DE components** from System76 (also Rust, also Iced-based). Their compositor and widget library could be leveraged.

### Compositor

We need a minimal Wayland compositor that:
1. Runs the shell UI fullscreen
2. Can overlay system dialogs (WiFi setup, pairing, etc.)
3. Handles input devices (touch, keyboard, mouse)
4. Manages screen power (DPMS)

Options:
- **cage** — single-application Wayland compositor. Dead simple. Perfect for kiosk.
- **smithay** — Rust Wayland compositor library. Build exactly what we need.
- **cosmic-comp** — System76's compositor. More features than we need but battle-tested.

Recommendation: Start with **cage** for the prototype (get the shell UI running fast), migrate to a custom **smithay**-based compositor for production (full control over transitions, blur, lock screen).

## System Architecture Diagram

```
                    ┌──────────────┐
                    │   SPEAKER    │
                    │  MICROPHONE  │
                    └──────┬───────┘
                           │ PipeWire
    ┌──────────────────────┼──────────────────────┐
    │                      │                       │
    │  ┌───────────────────▼────────────────────┐  │
    │  │          VOICE PIPELINE                 │  │
    │  │  Wake Word → VAD → STT → TTS           │  │
    │  └───────────────────┬────────────────────┘  │
    │                      │ text                   │
    │  ┌───────────────────▼────────────────────┐  │
    │  │        OPENCLAW GATEWAY                 │  │
    │  │  Agent · Tools · Messaging · Cron       │  │
    │  │  Browser · Nodes · Memory               │  │
    │  └───────────────────┬────────────────────┘  │
    │                      │ events                 │
    │  ┌───────────────────▼────────────────────┐  │
    │  │           SHELL UI                      │  │
    │  │  Ambient display · Cards · Viz          │  │
    │  │  Rust + Iced on Wayland                 │  │
    │  └────────────────────────────────────────┘  │
    │                                               │
    │              NIXOS BASE                       │
    │  Kernel · PipeWire · Network · Bluetooth      │
    └───────────────────────────────────────────────┘
```

## Update Strategy

```
Agent detects update available (cron or heartbeat)
  → Downloads new NixOS generation config
  → `nixos-rebuild switch` (atomic)
  → If boot fails → automatic rollback to previous generation
  → If boot succeeds → confirm and garbage-collect old generations
```

Users never run commands. The agent handles everything. If something goes wrong, NixOS automatically boots the last working generation.

## First Boot Experience

1. Device powers on → NixOS boots → Shell UI appears
2. Screen shows: OpenClaw logo + "Setting up..." animation
3. WiFi selection screen (if no Ethernet)
4. QR code appears: "Scan to pair your messaging"
   - Opens a setup page where user connects Telegram/WhatsApp/etc.
5. Agent speaks: "Hi, I'm ready. What would you like to call me?"
6. User names their agent via voice or text
7. Done. Agent is live.

Total setup time target: **under 3 minutes.**

## Security Model

- OpenClaw runs as unprivileged user `openclaw`
- System modifications go through NixOS rebuild (requires privilege escalation)
- Agent can request system changes but they go through a controlled NixOS module
- SSH is off by default, can be enabled through the agent
- Automatic security updates via unattended NixOS upgrades
- Full disk encryption (LUKS) optional during install
- No telemetry by default
