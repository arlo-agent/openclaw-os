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

### Test the NixOS Configuration

The NixOS config (`nix/`) defines the full OS image. Building it requires a Linux machine since NixOS targets Linux.

#### Option A: Build on a Linux machine (recommended)

On any x86_64 Linux box with Nix installed:

```bash
# Install Nix (if not already)
curl -L https://nixos.org/nix/install | sh
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf

# Build and run the VM
cd nix
nix build .#nixosConfigurations.openclaw-x86.config.system.build.vm
./result/bin/run-openclaw-x86-vm
```

#### Option B: Use UTM on macOS (no Nix needed)

For quick testing on Apple Silicon, run a stock NixOS VM in [UTM](https://mac.getutm.app/):

1. Download UTM from the App Store or https://mac.getutm.app/
2. Download the NixOS minimal ISO (aarch64): https://nixos.org/download#nixos-iso
3. Create a new VM in UTM → Linux → select the ISO → 4GB RAM, 20GB disk
4. Boot, install NixOS, then clone this repo inside the VM and apply the config:

```bash
# Inside the NixOS VM
nix-shell -p git
git clone https://github.com/arlo-agent/openclaw-os.git
cd openclaw-os/nix

# Apply the OpenClaw OS config
sudo nixos-rebuild switch --flake .#openclaw-x86
```

#### Option C: Remote Nix builder from macOS

If you want `nix build` to work from macOS, set up a remote Linux builder:

```bash
# In ~/.config/nix/nix.conf (or /etc/nix/nix.conf)
builders = ssh://your-linux-server x86_64-linux
```

See [NixOS Wiki: Distributed Builds](https://wiki.nixos.org/wiki/Distributed_build) for setup details.

> **Note:** For day-to-day shell UI development, `cargo run` in `shell/` is the fastest loop — runs natively on macOS, no VM needed. Use VM testing when you need to validate NixOS integration (services, boot, voice pipeline).

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
