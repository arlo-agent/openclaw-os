# OpenClaw Capability → Card Mapping

Every OpenClaw capability maps to a visual card type in the Shell UI. This is the complete map of what the agent can do and how each action surfaces visually.

## Communication Cards

### Message Card
**Capabilities:** `channels` · `message send/read` · `whatsapp` · `telegram` · `discord` · `signal` · `slack`

```
┌────────────────────────────────┐
│ 💬 Telegram · Francis          │
│ ─────────────────────────────  │
│ Hey, can you check the PR?     │
│                                │
│ [Reply]  [Read aloud]  [Mark]  │
│                     2 min ago  │
└────────────────────────────────┘
```

- Shows sender, channel icon, message preview
- Quick actions: reply (opens conversation), read aloud (TTS), mark as read
- Groups by channel when multiple unread
- Incoming messages trigger a card appearance + optional notification sound

### Conversation Card
**Capabilities:** `agent` · `sessions` · voice pipeline

```
┌────────────────────────────────┐
│ ◉ Speaking with you            │
│ ─────────────────────────────  │
│                                │
│ You: What's on my calendar?    │
│                                │
│ Agent: You've got two things   │
│ today — a standup at 10 and    │
│ a dentist at 3pm.              │
│                                │
│ [waveform visualization]       │
│                    ◉ Listening │
└────────────────────────────────┘
```

- Full-screen conversation mode when actively talking
- Voice waveform at bottom
- Text appears as agent speaks
- Previous messages fade/compact

### Email Card
**Capabilities:** custom integration (IMAP/SMTP)

```
┌────────────────────────────────┐
│ ✉️  Email · 3 unread           │
│ ─────────────────────────────  │
│ ⬤ John Smith                   │
│   Re: Project timeline         │
│ ⬤ GitHub                       │
│   [begin-wallet] PR #18 merged │
│ ○ Newsletter                   │
│   Weekly Cardano digest        │
│                                │
│ [Read first]  [Summary]        │
└────────────────────────────────┘
```

## Information Cards

### Weather Card
**Capabilities:** `weather` skill · web_search

```
┌────────────────────────────────┐
│ 🌤️  Dublin · 14°C              │
│ ─────────────────────────────  │
│                                │
│ Partly cloudy                  │
│ H: 16°  L: 9°                 │
│                                │
│ ▁▂▃▅▇▅▃▂▁  Rain at 4pm        │
│                                │
│ Tomorrow: ☀️ 18°C              │
└────────────────────────────────┘
```

- Integrates with ambient background (sky color shifts)
- Expandable to 5-day forecast
- Rain/event alerts auto-surface

### Calendar Card
**Capabilities:** calendar integration

```
┌────────────────────────────────┐
│ 📅 Today · Saturday Mar 15     │
│ ─────────────────────────────  │
│                                │
│ 10:00  Team standup     30min  │
│ 15:00  Dentist          1hr   │
│                                │
│ Tomorrow:                      │
│ 09:00  Sprint planning  1hr   │
│                                │
│ [Add event]  [This week]       │
└────────────────────────────────┘
```

- Upcoming events highlighted with countdown
- 2-hour warning auto-surfaces as alert card
- Can create events via voice

### Search Results Card
**Capabilities:** `web_search` · `web_fetch`

```
┌────────────────────────────────┐
│ 🔍 "NixOS Raspberry Pi 5"     │
│ ─────────────────────────────  │
│                                │
│ 1. NixOS Wiki — Pi Support     │
│    wiki.nixos.org/...          │
│                                │
│ 2. NixOS on Pi 5 Guide         │
│    blog.example.com/...        │
│                                │
│ 3. GitHub — nixos-hardware      │
│    github.com/NixOS/...        │
│                                │
│ [Read #1]  [Summarize all]     │
└────────────────────────────────┘
```

### Memory Card
**Capabilities:** `memory_search` · `memory_get`

```
┌────────────────────────────────┐
│ 🧠 Remembered                  │
│ ─────────────────────────────  │
│                                │
│ "Francis prefers Anthropic     │
│  models and rides an Aprilia   │
│  RS660"                        │
│                                │
│ Source: MEMORY.md#L42          │
│                     saved Feb  │
└────────────────────────────────┘
```

## System Cards

### Timer/Reminder Card
**Capabilities:** `cron` · reminders

```
┌────────────────────────────────┐
│ ⏰ Timer                       │
│ ─────────────────────────────  │
│                                │
│          18:42                  │
│                                │
│ "Check the oven"               │
│                                │
│ [+1 min]  [Stop]  [Snooze]    │
└────────────────────────────────┘
```

- Pins to ambient view (visible from across room)
- Alarm sound + voice announcement when triggered
- Large, readable countdown

### System Status Card
**Capabilities:** `gateway` · `status` · `doctor`

```
┌────────────────────────────────┐
│ ⚙️  System                     │
│ ─────────────────────────────  │
│                                │
│ Gateway: ● Online              │
│ Model: Claude Opus 4           │
│ WiFi: HomeNetwork ● Connected  │
│ Memory: 1.2GB / 4GB           │
│ Uptime: 3 days                │
│                                │
│ Update available: 2026.3.13    │
│ [Update now]  [Later]          │
└────────────────────────────────┘
```

### Update Card
**Capabilities:** `gateway update` · NixOS rebuild

```
┌────────────────────────────────┐
│ 🔄 Update Available            │
│ ─────────────────────────────  │
│                                │
│ OpenClaw 2026.3.8 → 2026.3.13 │
│                                │
│ Changes:                       │
│ • Fixed browser connector      │
│ • New voice pipeline options   │
│ • Bug fixes                    │
│                                │
│ [Install]  [Release notes]     │
└────────────────────────────────┘
```

### Network Card
**Capabilities:** NetworkManager · system

```
┌────────────────────────────────┐
│ 📶 Network                     │
│ ─────────────────────────────  │
│                                │
│ WiFi: HomeNetwork              │
│ Signal: ████░ Good             │
│ IP: 192.168.1.42              │
│ Speed: 85 Mbps ↓ / 12 Mbps ↑  │
│                                │
│ [Disconnect]  [Other networks] │
└────────────────────────────────┘
```

### Bluetooth Card
**Capabilities:** Bluetooth stack

```
┌────────────────────────────────┐
│ 🔵 Bluetooth                   │
│ ─────────────────────────────  │
│                                │
│ Connected:                     │
│ 🔊 JBL Flip 6                 │
│                                │
│ Available:                     │
│ 🎧 AirPods Pro                │
│ ⌨️ Logitech K380              │
│                                │
│ [Pair new]                     │
└────────────────────────────────┘
```

## Development / Power User Cards

### Code Card
**Capabilities:** `exec` · `read` · `write` · `edit`

```
┌────────────────────────────────┐
│ 💻 Code                        │
│ ─────────────────────────────  │
│                                │
│ ┌──────────────────────────┐   │
│ │ fn main() {              │   │
│ │     println!("Hello");   │   │
│ │ }                        │   │
│ └──────────────────────────┘   │
│                                │
│ Running... ✓ Success           │
│ Output: Hello                  │
│                                │
│ [Copy]  [Run again]            │
└────────────────────────────────┘
```

### GitHub Card
**Capabilities:** `github` skill · `gh` CLI

```
┌────────────────────────────────┐
│ 🐙 GitHub                      │
│ ─────────────────────────────  │
│                                │
│ PR #18 merged ✅                │
│ "Add Solana wallet page"       │
│ begin-wallet/begin-website     │
│                                │
│ 2 new issues in begin-core     │
│ CI: All checks passing ✓      │
│                                │
│ [View PR]  [Issues]            │
└────────────────────────────────┘
```

### Browser Card
**Capabilities:** `browser` · Chrome connector

```
┌────────────────────────────────┐
│ 🌐 Browser                     │
│ ─────────────────────────────  │
│                                │
│ [Live page preview/screenshot] │
│                                │
│ docs.openclaw.ai/getting-sta.. │
│                                │
│ [Open full]  [Screenshot]      │
└────────────────────────────────┘
```

## Device / IoT Cards

### Camera Card
**Capabilities:** `nodes camera_snap` · `camera_list`

```
┌────────────────────────────────┐
│ 📷 Camera · Front Door         │
│ ─────────────────────────────  │
│                                │
│ [Live preview / last snapshot] │
│                                │
│ Last motion: 14 min ago        │
│                                │
│ [Snap]  [Clip]  [All cameras]  │
└────────────────────────────────┘
```

### Node Device Card
**Capabilities:** `nodes` · `devices`

```
┌────────────────────────────────┐
│ 📱 Devices                     │
│ ─────────────────────────────  │
│                                │
│ 📱 iPhone · Francis            │
│   📍 Home · Battery 67%        │
│                                │
│ 💻 MacBook Pro                 │
│   🟢 Online                    │
│                                │
│ [Notify]  [Location]           │
└────────────────────────────────┘
```

### Screen Record Card
**Capabilities:** `nodes screen_record`

```
┌────────────────────────────────┐
│ 🖥️  Screen · MacBook           │
│ ─────────────────────────────  │
│                                │
│ [Screen capture preview]       │
│                                │
│ [Record clip]  [Screenshot]    │
└────────────────────────────────┘
```

## Media Cards

### Now Playing Card
**Capabilities:** media integration

```
┌────────────────────────────────┐
│ 🎵 Now Playing                 │
│ ─────────────────────────────  │
│                                │
│ Evanescence — Bring Me to Life │
│ ━━━━━━━━━━━━━━━━━━━━━━━▷──── │
│ 2:47 / 3:58                   │
│                                │
│ [⏮]  [⏸]  [⏭]  🔊 ━━━━━━━  │
└────────────────────────────────┘
```

- Integrates with ambient display (album art influences background)
- Voice controls: "pause", "next", "play [song]"

## Action / Confirmation Cards

### Confirmation Card
**Capabilities:** any destructive or external action

```
┌────────────────────────────────┐
│ ⚡ Confirm Action              │
│ ─────────────────────────────  │
│                                │
│ Send this email to             │
│ john@example.com?              │
│                                │
│ Subject: Project update        │
│ "Hi John, here's the latest..." │
│                                │
│ [Send ✓]  [Edit]  [Cancel ✗]  │
└────────────────────────────────┘
```

### Error Card
**Capabilities:** error handling

```
┌────────────────────────────────┐
│ ⚠️  Heads Up                   │
│ ─────────────────────────────  │
│                                │
│ Couldn't connect to Telegram.  │
│ The bot token might have       │
│ expired.                       │
│                                │
│ [Reconnect]  [Help]  [Dismiss] │
└────────────────────────────────┘
```

## Card Behavior Rules

### Priority & Stacking
1. **Critical:** Error, confirmation → always on top, require interaction
2. **High:** Messages, calendar alerts → stack, auto-dismiss after 30s
3. **Medium:** Search results, info cards → stack, auto-dismiss after 60s
4. **Low:** System status, weather → ambient overlay, persist

### Transitions
- **Appear:** Spring animation from bottom (300ms)
- **Dismiss:** Swipe left/right (gesture) or fade out (auto/voice)
- **Stack:** Cards push up as new ones arrive, max 3 visible
- **Expand:** Tap to expand with smooth height animation

### Voice Interaction
Every card can be interacted with by voice:
- "Read that message" → TTS reads the message card content
- "Reply: I'll check it now" → sends reply, dismisses card
- "Dismiss" → removes top card
- "Show me more" → expands card or shows full list
- "What was that notification?" → re-reads last dismissed card

## Capability Coverage Summary

| OpenClaw Feature | Card Type | Status |
|---|---|---|
| Messaging (Telegram, WA, Discord, etc.) | Message Card | Core |
| Voice conversation | Conversation Card | Core |
| Web search | Search Results Card | Core |
| Reminders / Cron | Timer Card | Core |
| System status | System Status Card | Core |
| Updates | Update Card | Core |
| Weather | Weather Card | Skill |
| Calendar | Calendar Card | Integration |
| Email | Email Card | Integration |
| GitHub (PRs, issues, CI) | GitHub Card | Skill |
| Browser | Browser Card | Core |
| Camera / Nodes | Camera Card | Core |
| Device management | Node Device Card | Core |
| Code execution | Code Card | Core |
| Media playback | Now Playing Card | Integration |
| Memory recall | Memory Card | Core |
| File operations | Code Card (reuse) | Core |
| Confirmations | Confirmation Card | Core |
| Errors / Warnings | Error Card | Core |
| Network | Network Card | System |
| Bluetooth | Bluetooth Card | System |
| Screen recording | Screen Record Card | Core |
