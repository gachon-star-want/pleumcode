---
title: "Use Pleum with Your AI Subscription"
description: "A quick update on using subscriptions for claude, gemini and codex"
authors: 
    - mic
---

You can use your subscriptions for codex, claude and gemini now with pleum, thanks to ACP! (Agent Client Protocol).
Codex is also special in that you can login directly to chatgpt - nothing else needs to be installed. 

Gemini now works via OAuth — just sign in with your Google account. At the time of writing, claude requires just one utility installed just once.

<!--truncate-->

## Why subscriptions?

Well you can use what you already pay for. Obviously! and sessions and so on are still in pleum. 
ACP gives a deeper connection to these agents than using the CLI as providers. In this world - you can think of this as a stack of agents: 
pleum plugs into gemini via ACP (and other things, clients could plug in to pleum!) but gemini (and also claude code) also act as an agent loop somewhat. 
With ACP you are using the tools that are (mostly) in the underlying agent. Codex, however, is a full power LLM api, so you can use extensions natively in pleum for that one.

## Claude Code — via ACP

If you have a Claude Code subscription, you can use it through pleum via the [Agent Client Protocol (ACP)](https://agentclientprotocol.com/). This requires installing a small adapter package:

```bash
npm install -g @zed-industries/claude-agent-acp
```

Then configure pleum to use it via the claude acp extension (CLI or GUI)


Or set it via environment variables:

```bash
export PLEUM_PROVIDER=claude-acp
pleum
```

pleum passes your MCP extensions through to Claude via ACP, so any custom MCP servers you've configured in pleum are available to the agent.

## ChatGPT — sign in with your account

If you have ChatGPT Plus or Pro, the `chatgpt_codex` provider lets you use pleum with your existing account. Just pick ChatGPT when you are setting up the pleum app for the first time (or changing to that provider)

The first time you run it, pleum will open a browser window for you to sign in with your ChatGPT account. After that, your session is cached locally.

The recommended model is `gpt-5.3-codex`, which is the default. You can also select `gpt-5.4` (OpenAI's latest omni model) or `gpt-5.2-codex` from the model picker.

## Gemini — via OAuth

If you have a Google account with Gemini access, the `Gemini` (`gemini_oauth`) provider lets you use pleum with your existing account. Just pick Gemini when setting up pleum for the first time (or changing providers).

The first time you run it, pleum will open a browser window for you to sign in with your Google account. After that, your session is cached locally.

## What about the old CLI providers?

Pleum previously supported `claude-code`, `codex`, and `gemini-cli` as "pass-through" CLI providers. These will be removed soon as ACP is the future! 

## Quick reference

| Subscription | Provider | Install | Extensions |
|---|---|---|---|
| Claude Code | `claude-acp` | `npm install -g @zed-industries/claude-agent-acp` | ✅ via MCP |
| ChatGPT Plus/Pro | `chatgpt_codex` | Nothing — OAuth sign-in | ✅ via MCP |
| Gemini | `gemini_oauth` | Nothing — OAuth sign-in | ✅ native |

Pick the one that matches what you're already paying for, and you're good to go.
