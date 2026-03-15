# Why Not Just a Web Dashboard?

A question that will come up: "Why build a custom shell in Rust? Why not just run Chrome in kiosk mode pointing at a web dashboard?"

## The Answer

### 1. Boot Time

Web approach: NixOS boots → Compositor starts → Chrome launches → Page loads → JS hydrates → Websocket connects → UI ready.
**Total: 8-15 seconds.**

Native approach: NixOS boots → Compositor + Shell start as one process → UI ready.
**Total: 3-5 seconds.**

For a device that should feel like it's always on, 15 seconds of Chrome loading is unacceptable.

### 2. Memory

Chrome on a Raspberry Pi 5 (4GB RAM):
- Chrome process: ~300-500MB
- GPU process: ~100MB
- Renderer: ~200MB
- **Total: ~600MB-800MB for a single page**

Iced/Rust shell:
- Single process: ~30-50MB
- GPU acceleration via wgpu
- **Total: ~50MB**

That's 15x less memory. On a Pi, this is the difference between snappy and swapping.

### 3. Animation Quality

Web: CSS animations, requestAnimationFrame, layout thrashing, GC pauses.
Reality: 60fps most of the time, but random jank from garbage collection, layout recalculation, or Chrome's internal scheduling. You feel it.

Native (Iced + wgpu): GPU-driven rendering, no GC, predictable frame timing.
Reality: Locked 60fps. Every frame. No exceptions.

Apple products feel good because they never drop frames. We need that.

### 4. System Integration

Web dashboard can't:
- Control screen brightness/DPMS
- Access PipeWire directly for audio routing
- Respond to hardware buttons
- Run below a compositor (it needs one)
- Manage Bluetooth pairing UI
- Show lock screen / boot animation

Native shell can do all of this because it IS the compositor layer.

### 5. Offline Behavior

Chrome needs to load. If the network is down, a web dashboard shows a blank page or cached stale content.

The native shell is always ready. It can show the ambient display, process voice commands (with local fallbacks), and gracefully indicate network status — all without loading anything.

### 6. The Feel

This is subjective but critical. Web apps feel like web apps. Even the best ones (Linear, Figma) have a subtle "this is a browser tab" quality. You can feel the abstraction layer.

Native apps — done well — feel like they belong on the device. Like they ARE the device. That's what Apple achieves and what we need.

## When Web IS Appropriate

- **Setup wizard** — a web page for initial pairing is fine (user opens it on their phone)
- **Remote access** — managing the device from another computer, web is perfect
- **OpenClaw dashboard** — the existing dashboard is useful for power users
- **Content rendering** — if the agent needs to show a webpage, embed a web view

The web is a tool in the toolkit. It's just not the shell.

## The Compromise Path

If native Rust/Iced proves too ambitious initially:

**Phase 1:** Use Tauri (Rust + WebView) — web tech for UI, but bundled as a native app. Smaller than Chrome, better system integration. Ship something.

**Phase 2:** Migrate critical UI components (ambient display, voice visualization, cards) to native Iced. Keep web for complex content rendering.

**Phase 3:** Full native shell. Web view only for embedded browser when needed.

This way we ship fast without committing to Chrome-as-shell forever.
