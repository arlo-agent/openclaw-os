# Quick Setup — `openclaw onboard`

## Overview

The fastest path from fresh install to working agent. One command, a few prompts, done.

```bash
openclaw onboard --auth-choice ollama
```

This runs the interactive onboarding wizard with **Ollama** pre-selected as the AI provider — fully local, no API keys, no cloud dependency.

## What It Does

1. **Detects Ollama** — checks if Ollama is installed and running, offers to install if not
2. **Picks a model** — lists available local models, recommends a default (e.g. `llama3`, `mistral`)
3. **Names the agent** — prompts for a name (or generates one)
4. **Selects a voice** — picks from bundled offline TTS voices
5. **Connects messaging** (optional) — Telegram, WhatsApp, Discord, etc.
6. **Writes config** — generates `openclaw.json` + workspace files
7. **Starts the gateway** — agent is live

## Auth Choices

| Flag | Provider | Requires |
|------|----------|----------|
| `--auth-choice ollama` | Ollama (local) | Ollama installed |
| `--auth-choice anthropic` | Anthropic | API key |
| `--auth-choice openai` | OpenAI | API key |
| `--auth-choice openrouter` | OpenRouter | API key |

Ollama is the default for OpenClaw OS — everything runs on-device, zero cloud calls for inference.

## Example Session

```
$ openclaw onboard --auth-choice ollama

  ✨ OpenClaw Quick Setup

  Checking Ollama... ✓ Running (v0.6.2)
  Available models:
    1. llama3.3 (8B) — recommended
    2. mistral (7B)
    3. deepseek-r1 (14B)

  Which model? [1]: 1

  What should your agent be called? [Atlas]: Nova

  Choose a voice:
    1. Calm (male)
    2. Warm (female)
    3. Clear (female)
    4. Deep (male)

  Voice? [2]: 2

  Connect messaging? (y/n) [n]: n

  ✓ Config written to ~/.openclaw/openclaw.json
  ✓ Agent "Nova" is ready

  Start with: openclaw gateway start
  Or just say: "Hey Nova"
```

## On OpenClaw OS

On a dedicated OpenClaw OS device, `openclaw onboard` runs automatically on first boot as part of the [first boot wizard](first-boot.md). The `--auth-choice` flag maps to Screen 6 (API Keys) — selecting Ollama skips the API key input entirely.

## After Setup

The agent is immediately functional:
- Voice interaction (if mic/speaker connected)
- Text via connected messaging channels
- Web search, reminders, file management
- All inference stays local with Ollama
