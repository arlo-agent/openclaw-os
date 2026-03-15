# UI Design — The Visual Experience

## Design Philosophy

This isn't a desktop. There are no windows to manage. No taskbar. No system tray with 12 icons. 

The screen is a **canvas for the agent to communicate with you visually.** When there's nothing to show, it's beautiful. When there's information, it appears naturally and disappears when done.

Think: the love child of Apple's Dynamic Island, the Nothing Phone's Glyph interface, and an ambient smart display.

## Core Concepts

### 1. The Ambient State

When idle, the screen shows a living, breathing ambient display:

- **Time and date** — elegant, large typography (think Dieter Rams clock)
- **Subtle particle field** — generative art that shifts with time of day (warm tones at sunset, cool at night, bright in morning)
- **Status indicators** — tiny, tasteful dots that show connectivity, agent status, message count
- **Weather gradient** — the background subtly reflects current weather (overcast = muted, sunny = warm, rain = cool ripples)

The ambient state is what you see from across the room. It's art, not UI.

### 2. Cards

All information appears as **cards** that emerge from the ambient state. Cards have physics — they slide in, stack naturally, and can be dismissed with a swipe or voice command.

Card types:
- **Message card** — new message from Telegram/WhatsApp/etc.
- **Alert card** — calendar event, timer, reminder
- **Status card** — system update, network change, weather alert  
- **Media card** — currently playing audio, podcast
- **Action card** — agent asking for confirmation ("Should I send this email?")
- **Info card** — agent showing you something (search results, comparison table, chart)

Cards follow strict design rules:
- **Frosted glass** (backdrop blur) — content behind remains faintly visible
- **Rounded corners** — 16px radius, consistent everywhere
- **Shadow depth** — cards closer to you cast deeper shadows
- **Typography** — SF Pro or Inter, never more than 2 font sizes per card
- **Color** — each card type has a subtle accent color, never garish
- **Animation** — cards spring in (spring physics, not linear), fade out when dismissed

### 3. The Conversation View

When actively talking to the agent (voice or text), the screen transitions smoothly:

- Ambient particles pull back to edges
- A clean conversation area appears center-screen
- Agent responses appear as flowing text with subtle typewriter animation
- Voice input shows a waveform visualization (not a spinning circle)
- Code blocks, tables, images render beautifully inline

The conversation view is NOT a chat log. It's the current interaction. Previous messages fade and compress. The focus is always on NOW.

### 4. Voice Visualization

When the agent is listening or speaking, visual feedback is critical:

**Listening state:**
- Subtle waveform at bottom of screen
- Particle field responds to audio amplitude
- Gentle pulsing glow around screen edges

**Speaking state:**
- Text appears as the agent speaks
- Waveform shows the agent's voice
- Cards may appear alongside speech for visual context

**Thinking state:**
- Particles swirl gently (not a loading spinner)
- Subtle haptic-like screen pulse

### 5. The Dock

A minimal floating dock at the bottom, only visible on touch/mouse proximity:

- **Voice button** — tap to talk (alternative to wake word)
- **Text input** — pull up keyboard
- **Quick actions** — context-dependent (e.g., music controls when playing)

The dock auto-hides. It's not always visible. When you need it, it's there.

## Visual Language

### Color Palette

```
Background:     #0A0A0F (near-black, not pure black — OLED friendly)
Surface:        #1A1A2E (card backgrounds)
Glass:          rgba(255, 255, 255, 0.08) (frosted glass overlay)
Primary:        #6C63FF (OpenClaw purple — used sparingly)
Text Primary:   #FFFFFF (pure white on dark)
Text Secondary: #8888AA (muted, for metadata)
Success:        #4ADE80 (green, for confirmations)
Alert:          #FB923C (amber, for warnings)
Error:          #F87171 (red, for errors)
```

Dark mode only. No light mode. The device is an ambient presence, not a laptop screen.

### Typography

```
Display:    Inter Display, 72px — time, hero text
Heading:    Inter, 24px semibold — card titles
Body:       Inter, 16px regular — content
Caption:    Inter, 12px medium — metadata, timestamps
Mono:       JetBrains Mono, 14px — code blocks
```

### Motion

All animations use spring physics, not ease-in-out:
- **Stiffness:** 200 (default), 400 (snappy, for small elements)
- **Damping:** 20 (default), 30 (less bounce)
- **Mass:** 1.0

No animation longer than 400ms. Nothing should feel slow.

### Spacing

8px grid. Everything aligns to multiples of 8. No exceptions.

## Screen States

```
┌─────────────────────────────────────────┐
│                                         │
│          AMBIENT STATE                  │
│                                         │
│    Beautiful generative background      │
│    Time / Date / Weather gradient       │
│    Status dots (connectivity, etc.)     │
│                                         │
│                                    ·──  │ ← Dock hint (line)
└─────────────────────────────────────────┘
         │                    │
    voice/touch          notification
         ↓                    ↓
┌──────────────────┐ ┌──────────────────┐
│ CONVERSATION     │ │ CARD OVERLAY     │
│                  │ │                  │
│ Active agent     │ │ Notification     │
│ interaction      │ │ card(s) appear   │
│                  │ │ over ambient     │
│ Text + voice     │ │                  │
│ visualization    │ │ Auto-dismiss or  │
│                  │ │ swipe away       │
│ [Dock visible]   │ │                  │
└──────────────────┘ └──────────────────┘
         │
    "show me X"
         ↓
┌──────────────────┐
│ FOCUS VIEW       │
│                  │
│ Full-screen      │
│ content:         │
│ - Browser        │
│ - Camera feed    │
│ - Document       │
│ - Media player   │
│                  │
│ "go back" to     │
│ return           │
└──────────────────┘
```

## Interaction Patterns

### Voice Commands (Natural Language)

No command syntax. Just talk:

- "What's on my calendar today?" → Cards appear with events
- "Play some music" → Media card appears, audio plays
- "Turn the screen off" → Screen dims to sleep
- "Show me the weather this week" → Weather card expands to forecast
- "Read my messages" → Agent reads unread messages, cards appear
- "Set a timer for 20 minutes" → Timer card pins to ambient view
- "Good night" → Screen goes dark, notifications muted until morning

### Touch/Mouse (Secondary)

- **Swipe up** from bottom → Dock appears
- **Swipe left/right** on card → Dismiss
- **Tap card** → Expand for detail
- **Long press** → Quick actions menu
- **Two finger pull down** → Quick settings (WiFi, brightness, volume)

### Physical Buttons (Hardware-Dependent)

- **Power button** — screen on/off (not shutdown)
- **Volume** — system volume
- **Dedicated voice button** (if hardware supports) — push-to-talk

## Responsive Design

The shell must work across screen sizes:

| Device | Resolution | Layout |
|--------|-----------|--------|
| 7" touchscreen (Pi) | 1024×600 | Single column, larger touch targets |
| 10" tablet display | 1280×800 | Single column, comfortable spacing |
| Monitor (desk) | 1920×1080+ | Multi-column cards, more ambient space |
| TV/large display | 3840×2160 | Living room mode, huge typography |

## Accessibility

- **High contrast mode** — available via voice command
- **Font scaling** — agent adjusts based on user preference
- **Screen reader** — agent IS the screen reader
- **Reduced motion** — disable particle effects, use fades instead of springs
- **Color blind modes** — adjusted palette available

## Inspiration / References

- Apple Dynamic Island — contextual UI that adapts
- Nothing OS Glyph — subtle, beautiful notification system  
- Google Ambient Mode — clock/photo/weather display
- Teenage Engineering TP-7 — hardware UI done right
- Dieter Rams — "less, but better"
- Calm technology — technology that doesn't demand attention

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
