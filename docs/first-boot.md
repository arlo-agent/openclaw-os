# First Boot Experience

## Goal

A brand new user plugs in the device, powers on, and within 3 minutes has a fully configured AI agent that knows their name, speaks in a voice they chose, and is connected to their messaging.

No terminal. No config files. No "install Node.js first." Just a beautiful, guided conversation.

## Boot Sequence (Internal)

```
Power on
  → UEFI/BIOS
  → NixOS kernel loads (quiet, no text)
  → Plymouth splash (OpenClaw logo, minimal animation)
  → systemd brings up networking, PipeWire, gateway
  → Shell UI launches fullscreen
  → First-boot wizard activates
```

**Target: Power to wizard screen in under 15 seconds.**

## The Wizard (User Experience)

The wizard is NOT a traditional form. It's a conversation with the agent — using on-screen UI with big, beautiful touch-friendly controls. Voice input is available once a mic is detected.

### Screen 1: Welcome

```
┌─────────────────────────────────────────┐
│                                         │
│            [OpenClaw Logo]              │
│                                         │
│         Welcome to OpenClaw             │
│                                         │
│    Your AI assistant lives here.        │
│    Let's get you set up.                │
│                                         │
│          [ Get Started → ]              │
│                                         │
│    Language: English ▼                  │
└─────────────────────────────────────────┘
```

- Logo animation plays (subtle, 2 seconds)
- Language selector at bottom (auto-detected from locale if possible)
- Single button to proceed
- Background: dark gradient, calm

### Screen 2: Connect to Network

Only shown if no Ethernet detected.

```
┌─────────────────────────────────────────┐
│                                         │
│     Connect to WiFi                     │
│                                         │
│     ┌─────────────────────────────┐     │
│     │ 📶 HomeNetwork          🔒  │     │
│     │ 📶 NeighborWiFi         🔒  │     │
│     │ 📶 CoffeeShop           🔓  │     │
│     │ 📶 Other...                 │     │
│     └─────────────────────────────┘     │
│                                         │
│     Password: [________________]        │
│                                         │
│              [ Connect → ]              │
└─────────────────────────────────────────┘
```

- Auto-scans available networks
- Big touch targets for each network
- On-screen keyboard for password (or physical keyboard)
- Shows connection progress with animation
- Skip option if using Ethernet

### Screen 3: Name Your Agent

This is where it gets personal.

```
┌─────────────────────────────────────────┐
│                                         │
│     What would you like to              │
│     call your assistant?                │
│                                         │
│     ┌─────────────────────────────┐     │
│     │  Arlo                       │     │
│     └─────────────────────────────┘     │
│                                         │
│     Suggestions:                        │
│     [ Nova ]  [ Atlas ]  [ Sage ]       │
│     [ Echo ]  [ Iris ]   [ Max  ]       │
│                                         │
│              [ Next → ]                 │
└─────────────────────────────────────────┘
```

- Text input with blinking cursor
- Tap-able name suggestions (curated, gender-neutral options)
- The name gets stored in the agent's IDENTITY.md
- Microphone icon appears if mic detected ("or say a name")

### Screen 4: Choose a Voice

The key screen. Two columns — Male and Female — each with 3-4 pre-built voices.

```
┌─────────────────────────────────────────┐
│                                         │
│     Choose a voice for [Agent Name]     │
│                                         │
│     ┌──────────────┐ ┌──────────────┐   │
│     │   MALE       │ │   FEMALE     │   │
│     │              │ │              │   │
│     │ ○ Calm       │ │ ○ Warm       │   │
│     │   ▶ Preview  │ │   ▶ Preview  │   │
│     │              │ │              │   │
│     │ ○ Energetic  │ │ ○ Clear      │   │
│     │   ▶ Preview  │ │   ▶ Preview  │   │
│     │              │ │              │   │
│     │ ○ Deep       │ │ ○ Bright     │   │
│     │   ▶ Preview  │ │   ▶ Preview  │   │
│     └──────────────┘ └──────────────┘   │
│                                         │
│     [ Clone a voice... ] (advanced)     │
│                                         │
│              [ Next → ]                 │
└─────────────────────────────────────────┘
```

- Each voice has a preview button — plays a short sample sentence using the agent's name: "Hi, I'm [Name]. How can I help you today?"
- Selected voice highlighted with accent color
- **"Clone a voice" option** (advanced) — record 10-15 seconds of a voice sample, the system clones it using the offline TTS model
- Voice selection is stored in config and used for all TTS output

**Pre-built voices are generated at build time using the offline TTS model and shipped as optimized voice profiles. No cloud dependency for any default voice.**

### Screen 5: Connect Messaging (Optional)

```
┌─────────────────────────────────────────┐
│                                         │
│     Connect your messaging              │
│     (you can do this later)             │
│                                         │
│     ┌─────────────────────────────┐     │
│     │ 💬 Telegram          [Add]  │     │
│     │ 💬 WhatsApp          [Add]  │     │
│     │ 💬 Discord           [Add]  │     │
│     │ 💬 Signal            [Add]  │     │
│     │ 💬 Slack             [Add]  │     │
│     └─────────────────────────────┘     │
│                                         │
│     [ Skip for now ]  [ Next → ]        │
└─────────────────────────────────────────┘
```

- Each channel shows a QR code or token input flow
- Telegram: bot token + chat ID
- WhatsApp: QR code pairing (uses openclaw's built-in WhatsApp Web)
- Can skip and configure later via voice ("Connect my Telegram")

### Screen 6: API Keys

```
┌─────────────────────────────────────────┐
│                                         │
│     Almost there! [Agent Name]          │
│     needs an AI brain.                  │
│                                         │
│     Paste your API key:                 │
│                                         │
│     Provider:                           │
│     ( • ) Anthropic (recommended)       │
│     ( ) OpenAI                          │
│     ( ) OpenRouter (many models)        │
│                                         │
│     API Key:                            │
│     ┌─────────────────────────────┐     │
│     │ sk-ant-...                  │     │
│     └─────────────────────────────┘     │
│                                         │
│     Don't have one? [Get a key →]       │
│     (shows QR to docs page)             │
│                                         │
│              [ Next → ]                 │
└─────────────────────────────────────────┘
```

- QR code option: scan to open docs on phone, paste key from phone
- "Get a key" shows a QR linking to a simple guide
- Validate key on entry (test API call, show green checkmark)

### Screen 7: Ready

```
┌─────────────────────────────────────────┐
│                                         │
│          ✨ All set!                    │
│                                         │
│    [Agent Name] is ready.               │
│                                         │
│    Try saying:                          │
│    "Hey [Name], what can you do?"       │
│                                         │
│    Or tap the mic to start talking.     │
│                                         │
│                                         │
│          [ Start using → ]              │
│                                         │
└─────────────────────────────────────────┘
```

- Ambient display starts fading in behind the card
- Agent speaks its first words: "Hey! I'm [Name]. Ready when you are."
- Tapping the button transitions to the ambient state
- First boot wizard is done and never shown again

## Configuration Storage

Everything from the wizard gets written to:

```
/home/openclaw/.openclaw/
├── openclaw.json          # Gateway config (API keys, channels, model)
└── workspace/
    ├── IDENTITY.md        # Agent name, personality seed
    ├── SOUL.md            # Generated personality (based on voice/name choice)
    └── voice/
        ├── config.json    # Selected voice profile, wake word
        └── profile.bin    # Voice model weights (for selected voice)
```

## Post-Setup Agent Capabilities

After first boot, the agent can:
- Respond to voice and text
- Send/receive messages on connected channels
- Answer questions, search the web, set reminders
- Control the device (WiFi, Bluetooth, display, volume)
- Learn the user's preferences over time

The agent should proactively offer guidance in the first 24 hours:
- "Would you like me to check the weather each morning?"
- "I can read your emails if you connect an account — just say 'set up email'"
- "Try saying 'good night' when you're done for the day"
