# OpenClaw OS

An AI-native operating system built on NixOS. Your agent isn't an app — it's the entire experience.

> **Status:** Research & Design Phase

## What is this?

OpenClaw OS is a purpose-built operating system where your AI agent is the primary interface. Not a chatbot window floating on a desktop. Not a web dashboard in a browser tab. The agent IS the operating system experience.

Think of it as what would happen if Apple designed an OS where Siri actually worked — and was the entire point.

## Why?

OpenClaw already integrates with messaging, browsers, cameras, calendars, files, code, and more. But it still runs as a service on someone else's OS. That means:

- Users fight with systemd, npm, Node.js versions
- The agent has to work around the OS instead of with it
- No control over the boot experience, the shell, the visual layer
- Voice is bolted on, not native
- Updates require CLI knowledge

An OS changes all of that. Plug in a device, power on, and you're talking to your agent.

## Design Principles

1. **The agent is the shell.** No app launcher, no file manager, no settings panel. You talk to your agent (voice or text), and things happen.
2. **Apple-level polish.** Smooth 60fps animations, consistent typography, beautiful transitions. Every pixel is intentional.
3. **Voice-first, screen-second.** Always-on wake word. The screen shows you things when visual context helps, but voice is the primary input.
4. **Zero-config for users.** Boot → pair your messaging → done. NixOS handles reproducibility under the hood, but users never see Nix.
5. **Declarative everything.** The entire OS state lives in config files. Factory reset is just "apply the default config."

## Architecture

See [docs/architecture.md](docs/architecture.md) for the full technical design.

## Repo Structure

```
├── docs/               # Design documents and research
│   ├── architecture.md     # Technical architecture (4-layer system)
│   ├── ui-design.md        # Interface design & UX (Apple-level polish)
│   ├── voice.md            # Voice pipeline (always-on, wake word → TTS)
│   ├── first-boot.md       # Setup wizard (name agent, pick voice, connect)
│   ├── offline-tts.md      # Offline TTS research (Chatterbox, Kokoro, Qwen3)
│   ├── capability-cards.md # Every OpenClaw feature mapped to a card
│   ├── rust-iced-deep-dive.md  # Why Rust + Iced, full technical analysis
│   └── why-not-web.md      # Rationale for native over Chrome kiosk
├── nix/                # NixOS configuration
│   ├── flake.nix       # System flake
│   └── modules/        # Custom NixOS modules
├── shell/              # Custom shell UI (Rust + Iced)
├── voice/              # Voice pipeline
└── assets/             # Branding, icons, wallpapers
```

## Target Hardware

- **Primary:** Mini PCs (Intel NUC, Beelink, etc.) — $150-300 range
- **Secondary:** Raspberry Pi 5 — $80 all-in
- **Stretch:** Any x86_64 or aarch64 machine

## License

TBD

---

*Built by [NeuralSpark](https://neuralspark.io)*
