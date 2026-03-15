# OpenClaw OS

An AI-native operating system built on NixOS. Your agent isn't an app — it's the entire experience.

> **Status:** Active Development — Shell UI prototype in progress

## What is this?

OpenClaw OS is a purpose-built operating system where your AI agent is the primary interface. Not a chatbot window floating on a desktop. Not a web dashboard in a browser tab. The agent IS the operating system experience.

Think of it as what would happen if Apple designed an OS where Siri actually worked — and was the entire point.

## Quick Start

### Prerequisites

**Rust toolchain** (1.75+):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

**System dependencies:**

macOS:
```bash
brew install pkg-config
```

Linux (Ubuntu/Debian):
```bash
sudo apt install -y pkg-config libwayland-dev libxkbcommon-dev libvulkan-dev
```

NixOS:
```bash
cd nix && nix develop
```

### Build & Run

```bash
cd shell

# Debug build
cargo build

# Run the shell UI
cargo run

# Release build (optimized)
cargo build --release

# Check without building (useful on headless servers)
cargo check
```

### Cross-compile for Raspberry Pi

```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

### Test in a NixOS VM (QEMU)

**1. Install Nix (if not already installed):**

macOS / Linux:
```bash
# Official Nix installer (multi-user, recommended)
curl -L https://nixos.org/nix/install | sh

# Restart your terminal, then enable flakes:
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

> **macOS note:** The installer creates a Nix volume on APFS. It's non-destructive and can be fully uninstalled later with `nix-uninstall`.

**2. Install QEMU:**

macOS:
```bash
brew install qemu
```

Linux (Ubuntu/Debian):
```bash
sudo apt install -y qemu-system-x86 qemu-utils
```

**3. Build and run the VM:**

```bash
cd nix

# Build the VM image (first run downloads NixOS packages — may take a while)
nix build .#nixosConfigurations.openclaw-x86.config.system.build.vm

# Run it
./result/bin/run-openclaw-x86-vm
```

> **Tip:** On Apple Silicon Macs, QEMU will use emulation (no KVM), so the VM will be slower than native. For day-to-day UI development, `cargo run` in `shell/` is much faster.

## Design Principles

1. **The agent is the shell.** No app launcher, no file manager, no settings panel. You talk to your agent (voice or text), and things happen.
2. **Apple-level polish.** Smooth 60fps animations, consistent typography, beautiful transitions. Every pixel is intentional.
3. **Voice-first, screen-second.** Always-on wake word. The screen shows you things when visual context helps, but voice is the primary input.
4. **Zero-config for users.** Boot → pair your messaging → done. NixOS handles reproducibility under the hood, but users never see Nix.
5. **Declarative everything.** The entire OS state lives in config files. Factory reset is just "apply the default config."

## Architecture

Four layers, each replaceable and independently testable:

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

See [docs/architecture.md](docs/architecture.md) for the full technical design.

## Project Structure

```
├── README.md
├── docs/                   # Design documents and research
│   ├── architecture.md         # Technical architecture (4-layer system)
│   ├── ui-design.md            # Interface design & UX
│   ├── voice.md                # Voice pipeline design
│   ├── first-boot.md           # Setup wizard flow
│   ├── offline-tts.md          # Offline TTS research
│   ├── capability-cards.md     # OpenClaw features → cards
│   ├── rust-iced-deep-dive.md  # Why Rust + Iced
│   └── why-not-web.md          # Native vs Chrome kiosk rationale
├── shell/                  # Shell UI (Rust + Iced 0.13)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # App entry, TEA architecture
│       ├── theme.rs            # Colors, fonts, spacing constants
│       ├── ambient.rs          # Ambient state (clock, status dots)
│       ├── cards.rs            # Card system (message, alert, status, info)
│       ├── conversation.rs     # Conversation view with typewriter effect
│       ├── dock.rs             # Floating dock (voice, text toggle)
│       └── widgets/
│           ├── mod.rs
│           ├── particle_field.rs   # Canvas-based particle animation
│           └── glass_card.rs       # Frosted glass container styling
├── nix/                    # NixOS configuration
│   ├── flake.nix               # System flake + dev shell
│   └── modules/                # Custom NixOS modules
├── voice/                  # Voice pipeline (planned)
└── assets/                 # Branding, icons, wallpapers
```

## Target Hardware

- **Primary:** Mini PCs (Intel NUC, Beelink, etc.) — $150-300 range
- **Secondary:** Raspberry Pi 5 — $80 all-in
- **Stretch:** Any x86_64 or aarch64 machine

## License

TBD

---

*Built by [NeuralSpark](https://neuralspark.io)*
