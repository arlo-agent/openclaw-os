# Offline TTS — ElevenLabs Quality Without the Cloud

## The Goal

Ship pre-built male and female voice profiles that sound as good as ElevenLabs, run 100% offline, and work on hardware as modest as a Raspberry Pi 5.

## Model Landscape (as of March 2026)

### Tier 1: Near-ElevenLabs Quality

#### Chatterbox (Resemble AI)
- **Quality:** Beat ElevenLabs in blind tests (63.8% listener preference)
- **Model size:** 0.5B parameters (Llama architecture)
- **Voice cloning:** Yes — from just 5 seconds of reference audio
- **Languages:** 23 languages
- **License:** Open source (check specific terms for commercial use)
- **Hardware needs:** GPU recommended for real-time (3060 Ti or better), CPU possible but slow
- **Latency:** Sub-200ms on GPU
- **Strengths:** Best cloning quality, emotional range, natural prosody
- **Weaknesses:** Turbo variant trades quality for speed (original model preferred)
- **Repo:** github.com/resemble-ai/chatterbox

#### Qwen3-TTS (Alibaba/Qwen)
- **Quality:** Competitive with ElevenLabs, very natural
- **Models:** 1.7B and 0.6B variants
- **Voice cloning:** Yes — provide reference audio + transcript
- **Languages:** Multi-language (strong on English, Chinese, Japanese, more)
- **License:** Open source
- **Hardware needs:** 0.6B runs on modest GPU; 1.7B needs 8GB+ VRAM
- **Features:** Free-form voice design (describe the voice you want in text), streaming output
- **Strengths:** Voice design from text description, good multilingual support
- **Weaknesses:** Larger model, newer (less community tooling)
- **Repo:** github.com/QwenLM/Qwen3-TTS

### Tier 2: Good Quality, Very Lightweight

#### Kokoro
- **Quality:** Best for its size — sounds great for 82M parameters
- **Model size:** 82M parameters (tiny!)
- **Voice cloning:** No — fixed set of pre-trained voices
- **Languages:** English primarily, some multilingual support
- **License:** Open source
- **Hardware needs:** Runs on CPU! Even a Pi can handle it
- **ONNX support:** Yes — optimized runtime, very fast inference
- **Strengths:** Tiny, fast, CPU-only, great for embedded/Pi
- **Weaknesses:** No voice cloning, limited voice variety, English-focused
- **Repo:** huggingface.co/hexgrad/Kokoro-82M

### Tier 3: Established but Older

#### Piper TTS
- **Quality:** Good, not great. Sounds "AI" to trained ears
- **Model size:** Various (VITS-based, small)
- **Voice cloning:** No — pre-trained voices only
- **Languages:** 30+ languages, many voices
- **License:** MIT
- **Hardware needs:** CPU only, runs anywhere including Pi Zero
- **Strengths:** Mature, stable, tons of voices, tiny resource footprint
- **Weaknesses:** Noticeably less natural than Chatterbox/Qwen3-TTS/Kokoro
- **Repo:** github.com/rhasspy/piper

## Recommended Strategy

### Pre-built Voice Profiles

Ship 6 voices (3 male, 3 female), each with a distinct personality:

| Voice | Gender | Character | Use Case |
|-------|--------|-----------|----------|
| **Calm** | Male | Warm, measured, reassuring | Default assistant |
| **Energetic** | Male | Upbeat, enthusiastic | Younger/tech audience |
| **Deep** | Male | Authoritative, rich bass | Professional/formal |
| **Warm** | Female | Friendly, conversational | Default assistant |
| **Clear** | Female | Precise, articulate | Information delivery |
| **Bright** | Female | Light, cheerful | Casual/fun interactions |

### Build Pipeline

```
1. Record 15-30s reference audio for each voice persona
   (use professional voice actors or high-quality samples)
        ↓
2. Generate voice profile using Chatterbox at build time
   (runs on build server with GPU, not on user device)
        ↓
3. Export optimized voice model/embeddings
        ↓
4. Ship as part of OpenClaw OS image
        ↓
5. At runtime, use voice profile for TTS inference
```

### Runtime Architecture (Tiered)

```
User device has GPU (mini PC with dedicated/integrated GPU)?
  → Use Chatterbox with pre-built voice profile
  → ElevenLabs-tier quality
  → Real-time streaming

User device is CPU-only (Pi 5, older hardware)?
  → Use Kokoro with closest matching pre-trained voice
  → Good quality, very fast on CPU
  → Fallback: Piper TTS for absolute minimum hardware

User has internet and prefers cloud?
  → Use ElevenLabs / OpenAI TTS APIs
  → Best quality, lowest latency
  → Configurable via voice settings
```

### Custom Voice Cloning (Advanced Feature)

For users who want a custom voice during setup:

```
1. User taps "Clone a voice" in setup wizard
2. Prompt: "Read this paragraph aloud" (shows text on screen)
3. Record 10-15 seconds of speech via device microphone
4. Run Chatterbox voice cloning on-device (takes 30-60s on GPU)
5. Preview: "Here's how I'll sound" (plays cloned voice)
6. User approves or tries again
7. Voice profile saved locally
```

This requires GPU hardware. On Pi/CPU-only devices, the option is hidden or shows "Requires GPU hardware."

## Model Distribution

- **Default voices:** Bundled in the OS image (~500MB for Chatterbox model + 6 voice profiles)
- **Kokoro fallback:** Additional ~100MB, always included
- **Piper emergency fallback:** ~50MB, always included
- **Custom voice profiles:** Generated on-device, stored in user config
- **Model updates:** Delivered via NixOS system updates (new/improved voices)

## Quality Comparison

Informal ranking based on community benchmarks and blind tests:

```
ElevenLabs (cloud)    ████████████████████ 10/10
Chatterbox (local)    ███████████████████  9.5/10
Qwen3-TTS 1.7B       ██████████████████   9/10
Kokoro 82M            ████████████████     8/10
Piper                 ████████████         6/10
espeak                ██████               3/10
```

## Open Questions

1. **Licensing:** Chatterbox license terms for commercial redistribution need legal review
2. **Model fine-tuning:** Can we fine-tune Chatterbox on curated voice data for even better preset voices?
3. **Streaming:** Chatterbox and Qwen3-TTS both support streaming — need to benchmark actual latency on target hardware
4. **Emotional control:** Chatterbox supports emotion parameters — should the agent adjust tone based on context? (urgent notification = serious tone, joke = playful)
5. **Pi 5 performance:** Need to benchmark Kokoro on Pi 5 — is it fast enough for real-time conversation?
