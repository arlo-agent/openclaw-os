# Voice Pipeline Design

## Overview

Voice is the primary input method. It must feel instant, natural, and always available.

The pipeline runs entirely locally for wake word detection. Cloud services are used for STT/TTS quality, with local fallbacks available.

## Pipeline Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    AUDIO INPUT                           │
│  PipeWire source → Ring buffer (5s rolling)              │
└──────────────────────┬──────────────────────────────────┘
                       │ continuous audio stream
                       ▼
┌─────────────────────────────────────────────────────────┐
│              WAKE WORD DETECTION                         │
│  Engine: OpenWakeWord or Porcupine                       │
│  Always-on, <1% CPU, runs on dedicated thread            │
│  Custom wake word: configurable ("Hey Claw", agent name) │
└──────────────────────┬──────────────────────────────────┘
                       │ wake word detected
                       ▼
┌─────────────────────────────────────────────────────────┐
│         VOICE ACTIVITY DETECTION (VAD)                   │
│  Engine: Silero VAD (local, ~2MB model)                  │
│  Detects speech start/end                                │
│  Handles pauses (don't cut off mid-thought)              │
│  Timeout: 2s of silence = end of utterance               │
└──────────────────────┬──────────────────────────────────┘
                       │ speech segment (audio bytes)
                       ▼
┌─────────────────────────────────────────────────────────┐
│           SPEECH-TO-TEXT (STT)                            │
│                                                          │
│  Primary: Whisper API (OpenAI) — fastest, most accurate  │
│  Alt cloud: Deepgram — streaming, lower latency          │
│  Local fallback: whisper.cpp (runs on device)            │
│                                                          │
│  Streaming mode: partial results shown as user speaks     │
└──────────────────────┬──────────────────────────────────┘
                       │ transcript (text)
                       ▼
┌─────────────────────────────────────────────────────────┐
│            OPENCLAW AGENT                                │
│  Process as normal text input                            │
│  Agent doesn't know/care if input was voice or text      │
│  Response text is generated                              │
└──────────────────────┬──────────────────────────────────┘
                       │ response text
                       ▼
┌─────────────────────────────────────────────────────────┐
│           TEXT-TO-SPEECH (TTS)                            │
│                                                          │
│  Primary: ElevenLabs — best quality, emotional range     │
│  Alt: OpenAI TTS — good quality, lower cost              │
│  Local fallback: Piper TTS (runs on device, decent)      │
│                                                          │
│  Streaming: start speaking before full response is ready │
│  Sentence-level chunking for natural pacing              │
└──────────────────────┬──────────────────────────────────┘
                       │ audio stream
                       ▼
┌─────────────────────────────────────────────────────────┐
│                   AUDIO OUTPUT                           │
│  PipeWire sink → Speaker / Bluetooth / HDMI              │
│  Echo cancellation (avoid agent hearing itself)          │
│  Ducking: lower media volume while agent speaks          │
└─────────────────────────────────────────────────────────┘
```

## Latency Targets

| Stage | Target | Notes |
|-------|--------|-------|
| Wake word detection | <100ms | Local, always-on |
| VAD end-of-speech | <300ms | Silero is fast |
| STT | <500ms | Streaming helps |
| Agent thinking | Varies | Show "thinking" state |
| TTS first audio | <300ms | Stream first sentence |
| **Total wake-to-first-audio** | **<1.5s** | After agent response |

## Wake Word

### Option A: OpenWakeWord (Recommended)

- Open source, MIT license
- Custom wake words trainable with minimal data
- Runs on CPU, ~0.5% utilization
- Python-based but can run as a lightweight service
- No cloud dependency

### Option B: Porcupine (Picovoice)

- Commercial but free tier available
- Pre-trained wake words ("Hey Google" quality)
- Custom wake words available on paid plan
- C library, very efficient
- Runs on everything including Pi Zero

### Wake Word Configuration

Users choose their wake word during setup. Options:
- Agent's name (e.g., "Hey Arlo")
- "Hey Claw" (default)
- Custom phrase (requires training, takes ~5 minutes of samples)

## Echo Cancellation

Critical problem: the agent's voice output gets picked up by the microphone, creating a feedback loop.

Solution:
1. **Software AEC** — PipeWire has echo cancellation modules
2. **Reference signal** — feed the TTS output as reference to the AEC
3. **Mute during playback** — simplest approach, mute mic while agent speaks
4. **Hardware** — recommend USB speakerphones with built-in AEC (e.g., Jabra Speak series)

Recommended approach: software AEC via PipeWire + reference signal. Fall back to mute-during-playback if AEC quality is poor.

## Barge-In Support

Users should be able to interrupt the agent mid-speech:

1. Wake word detected during TTS playback
2. TTS immediately stops
3. New utterance is captured
4. Agent processes the interruption

This requires the wake word detector to remain active during TTS output, which means good echo cancellation is essential.

## Multi-Room Audio (Future)

If multiple OpenClaw OS devices exist in a home:
- Wake word activates the nearest device (loudest input)
- Audio can be routed to specific rooms
- "Play music in the kitchen" → routes to kitchen device

## Offline Mode

When internet is unavailable, the voice pipeline degrades gracefully:

| Component | Online | Offline |
|-----------|--------|---------|
| Wake word | Local ✓ | Local ✓ |
| VAD | Local ✓ | Local ✓ |
| STT | Cloud (Whisper API) | Local (whisper.cpp) — slower, less accurate |
| Agent | Cloud (LLM API) | Limited — can still do local commands |
| TTS | Cloud (ElevenLabs) | Local (Piper) — less natural |

The agent should tell the user: "I'm offline right now, so I might be a bit slower."

## Privacy

- Wake word detection is ALWAYS local. No audio leaves the device until wake word is confirmed.
- Audio is never stored on disk by default.
- Optional: user can enable conversation logging for memory/improvement.
- STT audio is sent to cloud providers only after wake word activation and during active speech.
- No ambient listening. No always-recording. No "review your recordings" nightmare.

## Hardware Recommendations

### Minimum (Raspberry Pi 5)
- Built-in audio jack for output
- USB microphone (e.g., ReSpeaker USB Mic Array — has onboard AEC)
- Works but STT/TTS must be cloud

### Recommended (Mini PC)
- USB speakerphone (Jabra Speak 410/510) — built-in speaker, mic, AEC
- Or: good USB mic + powered speakers
- Can run local whisper.cpp for offline STT

### Premium (Custom Hardware — Future)
- Custom PCB with far-field microphone array
- Dedicated DSP for echo cancellation
- High-quality speaker driver
- Industrial design case (think Teenage Engineering aesthetic)
- This is the "sell hardware" play if it gets that far
