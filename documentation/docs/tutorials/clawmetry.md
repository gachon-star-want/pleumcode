---
description: Observe your pleum sessions locally with ClawMetry, with no configuration
---

# Observability with ClawMetry

This tutorial covers how to use ClawMetry to see what your pleum sessions did, which models they used, and what they cost. Unlike the tracing integrations, there is nothing to configure: ClawMetry reads the session store pleum already writes.

## What is ClawMetry

[ClawMetry](https://clawmetry.com/) is an [open-source](https://github.com/vivekchand/clawmetry) (MIT) observability dashboard for AI agents. It runs on your machine, reads the local files your agent already produces, and serves a dashboard at `http://localhost:8900`. pleum is a free runtime in the open source app, and the pleum adapter ships in the package.

## Why ClawMetry for pleum

- **No instrumentation**: No environment variables, no exporter, no SDK. ClawMetry reads `sessions.db` directly, so past sessions show up too.
- **Local by default**: The dashboard runs on your machine and nothing is sent anywhere. Cloud sync exists but is opt-in and off unless you turn it on.
- **Read-only**: pleum owns its session store. ClawMetry always opens it read-only and never writes to it.
- **Real token counts**: pleum records usage on disk, so token totals come from your sessions rather than an estimate.
- **Open source**: MIT licensed, and the pleum adapter is in the repository you can read.

## Set up ClawMetry

```bash
pip install clawmetry
clawmetry
```

Then open `http://localhost:8900`.

That is the whole setup. There is no pleum-side configuration step, because ClawMetry does not sit in the request path.

## Run pleum

Use pleum exactly as you normally would:

```bash
pleum session
```

ClawMetry auto-detects the [session store](/docs/guides/logs#session-records) by resolving pleum's data directory the same way pleum does:

| Platform | Session store |
| --- | --- |
| macOS and Linux | `$XDG_DATA_HOME/pleum/sessions/sessions.db`, defaulting to `~/.local/share/pleum/sessions/sessions.db` |
| Windows | `%APPDATA%\Block\pleum\data\sessions\sessions.db` |

If [`PLEUM_PATH_ROOT`](/docs/guides/environment-variables) is set, ClawMetry reads `$PLEUM_PATH_ROOT/data/sessions/sessions.db` instead, on every platform. On macOS it also checks `~/Library/Application Support/Block/pleum/` last, so an older install that still keeps its data there is picked up.

Sessions you ran before installing ClawMetry appear as well.

## What you see

- **Sessions**: every pleum session with its start time, message count, and working directory.
- **Transcripts**: the full turn by turn conversation, including tool calls and their results.
- **Models**: which model each session used, and how usage is split across them.
- **Tokens and cost**: input, output, and total tokens per session, with cost where pleum recorded it.

:::note
pleum populates a cost figure only for providers that report one. With a local provider such as Ollama there is no cost to record, so ClawMetry shows the token counts and leaves cost empty rather than inventing a number.
:::

:::tip
If you run several agents, the runtime switcher at the top of the dashboard scopes every view to pleum alone.
:::

## Learn more

- [ClawMetry repository](https://github.com/vivekchand/clawmetry)
- [The pleum adapter source](https://github.com/vivekchand/clawmetry/blob/main/clawmetry/adapters/pleum.py)
- [ClawMetry documentation](https://clawmetry.com/docs)
