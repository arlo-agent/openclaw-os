# UI Design — The Visual Experience

## Design Philosophy

This isn't a desktop. There are no windows to manage. No taskbar. No system tray with 12 icons.

The screen is a **canvas for the agent to communicate with you visually.** When there's nothing to show, it's beautiful. When there's information, it appears naturally and disappears when done.

Think: the love child of Apple's Dynamic Island, the Nothing Phone's Glyph interface, and an ambient smart display.

## Core Concepts

### 1. The Ambient State

When idle, the screen shows a living, breathing ambient display:

- **Time and date** — elegant, large typography
- **Generative aurora background** — organic flowing gradients driven by simplex noise (see below)
- **Status indicators** — tiny, tasteful dots that show connectivity and agent status
- **Notification cards** — slide in from the right when events arrive

The ambient state is what you see from across the room. It's art, not UI.

### 2. Generative Background

The background is a multi-layered generative art piece, not a static wallpaper:

- **Layer 1 — Aurora bands:** Wide color bands displaced by multi-octave OpenSimplex noise. Colors interpolate between coral (#ff4d4d) and cyan (#00e5cc) brand colors. Each band has its own noise offset for independent motion.
- **Layer 2 — Flow field curves:** A grid of control points defines a flow field. Curves trace through the field, creating organic texture lines.
- **Layer 3 — Drift particles:** ~120 small particles follow the flow field, glowing subtly as they drift. Their color is position-dependent, creating depth.
- **Layer 4 — Vignette:** Soft edge darkening for depth and focus.

The entire field has a "breathing" pulse — a slow sinusoidal modulation of brightness that makes it feel alive. Time evolution is slow (0.008 per frame) so the animation feels geological, not frenetic.

Performance: targets 60fps with canvas caching. All rendering uses iced's canvas widget.

**Theme adaptation:** In light mode, all layer opacities are reduced (~50%) to maintain readability against the lighter background.

### 3. Cards

All information appears as **cards** that emerge from the ambient state. Cards have physics — they slide in with spring-like decay animation.

Card types:
- **Message card** — new message from Telegram/WhatsApp/Discord
- **Alert card** — calendar event, timer, reminder
- **Status card** — system update, network change
- **Info card** — agent showing information

Cards follow strict design rules:
- **Frosted glass** — semi-transparent background using `surface_card_strong` from palette
- **Rounded corners** — 16px radius, consistent everywhere
- **Shadow depth** — cards cast deep shadows (blur 16px)
- **Typography** — Display (72px), Heading (24px), Body (16px), Caption (12px)
- **Color** — each card type has a subtle accent dot (coral for messages, cyan for status)
- **Animation** — cards spring in (offset decays by 12% per frame), no linear easing

### 4. The Dock

A glass-pill shaped dock anchored at the bottom center. **Always visible** (no auto-hide).

Layout: `[🎤] [________________text input________________] [→] [☀/🌙]`

- **Mic button** (left) — tap to toggle voice listening mode
- **Text input** (center) — always visible, ready to type. Placeholder: "Talk to your agent..."
- **Send button** (right) — arrow icon, activates when text is present. Coral color when active, muted when inactive.
- **Theme toggle** (far right) — sun/moon icon to switch between dark and light modes

Pressing Enter or clicking the send button submits the message. Messages go to the gateway (or mock) and responses appear in the conversation view.

The dock has full frosted glass treatment with extra-rounded corners (32px radius) and a deep shadow.

### 5. The Conversation View

When a message is sent, the screen transitions to conversation view:

- Scrollable message list with chat bubbles
- User messages: accent-bordered glass bubbles, right-aligned
- Agent messages: standard glass bubbles, left-aligned
- **Typewriter effect** — agent responses reveal character by character
- Text input in the conversation view syncs with the dock input

### 6. Light/Dark Mode

The shell supports both dark and light themes, toggled via the dock button.

**Dark theme (default)** — extracted from openclaw.ai CSS:
```
Background:     #050810 (deep navy-black)
Surface:        #0a0f1a
Elevated:       #111827
Coral Bright:   #ff4d4d
Coral Mid:      #e63946
Coral Dark:     #991b1b
Cyan Bright:    #00e5cc
Cyan Mid:       #14b8a6
Text Primary:   #f0f4ff
Text Secondary: #8892b0
Text Muted:     #5a6480
```

**Light theme:**
```
Background:     #fcfeff
Surface:        #ffffff
Elevated:       #f5f9ff
Coral Bright:   #ef4b58
Coral Mid:      #de3f4d
Coral Dark:     #c43645
Cyan Bright:    #008f87
Cyan Mid:       #00766e
Text Primary:   #0b1220
Text Secondary: #2e405c
Text Muted:     #5f7290
```

Both themes include matching semi-transparent surface colors for cards, overlays, and borders.

All components adapt to the current theme via `OpenClawPalette::from_mode()`. The generative background reduces opacity in light mode for readability.

## Gateway Integration

The shell communicates with the OpenClaw gateway for agent interactions.

### Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────┐
│  Shell UI (iced) │────▶│  Gateway Module   │────▶│  OpenClaw   │
│                  │◀────│  (gateway.rs)     │◀────│  Gateway    │
│  - Ambient view  │     │                   │     │  :3000      │
│  - Conversation  │     │  Mock mode:       │     └─────────────┘
│  - Cards         │     │  - Canned replies │
│  - Dock          │     │  - Fake notifs    │
└─────────────────┘     └──────────────────┘
```

### Modes

**Mock mode** (`--mock` flag or `OPENCLAW_MOCK=1`):
- Simulates agent responses with context-aware canned replies
- Generates periodic notification cards (every 30s)
- No network calls — good for development and demo

**Real mode** (default):
- Connects to `http://localhost:3000` (or `OPENCLAW_GATEWAY_URL`)
- Sends user messages via HTTP POST
- Receives agent responses and channel events
- Incoming notifications create typed cards

### Event Flow

1. User types in dock → `DockMessage::Submit`
2. App calls `gateway.send_message(text)`
3. Gateway produces `GatewayEvent::AgentResponse(text)`
4. On next tick, `drain_events()` processes all pending events
5. Agent responses become `ChatMessage` entries with typewriter effect
6. Notifications become `Card` entries that slide in

## Spacing & Typography

8px grid. Everything aligns to multiples of 8.

```
Display:    72px — time, hero text
Heading:    24px — card titles
Body:       16px — content
Caption:    12px — metadata, timestamps
```

Target fonts (from openclaw.ai):
- Display: "Clash Display"
- Body: "Satoshi"
- Mono: "SF Mono", "JetBrains Mono"

## Screen States

```
┌─────────────────────────────────────────┐
│                                         │
│          AMBIENT STATE                  │
│                                         │
│    Aurora generative background         │
│    Time / Date centered                 │
│    Status dots (top-left)               │
│    Notification cards (right panel)     │
│                                         │
│  [🎤] [__text input__] [→] [☀]         │ ← Dock (always visible)
└─────────────────────────────────────────┘
         │
    type + send
         ↓
┌──────────────────────────────────────────┐
│ CONVERSATION VIEW                        │
│                                          │
│ Scrollable chat bubbles                  │
│ User (right, accent) ←→ Agent (left)     │
│ Typewriter reveal on agent messages      │
│                                          │
│  [🎤] [__text input__] [→] [☀]          │
└──────────────────────────────────────────┘
```

## Anti-Patterns (Things We Will Never Do)

- ❌ System tray with 15 icons
- ❌ Notification badges with numbers
- ❌ Settings panels with 200 toggles
- ❌ Loading spinners
- ❌ Modal dialogs that block everything
- ❌ "Are you sure?" confirmations for non-destructive actions
- ❌ Hamburger menus
- ❌ Tutorial overlays on first boot
- ❌ App store / marketplace UI

## Inspiration / References

- Apple Dynamic Island — contextual UI that adapts
- Nothing OS Glyph — subtle, beautiful notification system
- Google Ambient Mode — clock/photo/weather display
- Teenage Engineering TP-7 — hardware UI done right
- Dieter Rams — "less, but better"
- Calm technology — technology that doesn't demand attention
- shadertoy.com aurora effects — for the generative background aesthetic
