---
title: Set up pleum in Zed
sidebar_position: 3
description: See how Zed installs and configures pleum as an ACP agent.
---

# Set up pleum in Zed

Zed can run pleum as an ACP agent. Install pleum from the ACP Registry for the
simplest setup, or configure it manually to use your own pleum binary and
environment overrides.

## Install pleum from the ACP Registry

Zed has built-in ACP Registry support, so it can download and run pleum for you
without manual configuration.

1. Open Zed
2. Open Agent Settings
3. Click `Add Agent`, then choose `Install from Registry`
4. Select `pleum`

A registry-installed pleum runs the same `pleum acp` server and reads your
existing pleum configuration, so your providers, models, and extensions carry
over. Zed keeps the installed version up to date for you.

## Configure pleum manually

Use a custom agent if you want to run your own pleum binary, such as a local
development build, or pass environment overrides.

### Prerequisites

Ensure you have both Zed and the pleum CLI installed:

- **Zed**: Download from [zed.dev](https://zed.dev/)
- **pleum CLI**: Follow the [installation guide](/docs/getting-started/installation)

Verify pleum is installed:

```bash
pleum --version
```

### Add pleum to your Zed settings

1. Open Zed
2. Open Agent Settings, click `Add Agent`, then choose `Add Custom Agent`. Zed
   scaffolds an `agent_servers` entry and opens your settings file
3. Edit the entry so it runs pleum:

```json
{
  "agent_servers": {
    "pleum": {
      "type": "custom",
      "command": "pleum",
      "args": ["acp"]
    }
  }
}
```

You can now interact with pleum directly in Zed. ACP sessions use the extensions
enabled in your pleum configuration, so their tools are also available in Zed.

## Override the provider and model

By default, pleum uses the provider and model defined in your
[configuration file](/docs/guides/config-files). Override them for a specific
agent configuration with the `PLEUM_PROVIDER` and `PLEUM_MODEL` environment
variables.

This example configures two pleum agents with different model settings:

```json
{
  "agent_servers": {
    "pleum": {
      "type": "custom",
      "command": "pleum",
      "args": ["acp"]
    },
    "pleum (GPT-4o)": {
      "type": "custom",
      "command": "pleum",
      "args": ["acp"],
      "env": {
        "PLEUM_PROVIDER": "openai",
        "PLEUM_MODEL": "gpt-4o"
      }
    }
  }
}
```

## Use Zed MCP servers with pleum

MCP servers in Zed's `context_servers` configuration are automatically
available to pleum. This lets native Zed features and the pleum agent use the
same MCP servers.

```json
{
  "context_servers": {
    "filesystem": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "/path/to/allowed/dir"
      ]
    }
  },
  "agent_servers": {
    "pleum": {
      "type": "custom",
      "command": "pleum",
      "args": ["acp"]
    }
  }
}
```

All MCP servers in `context_servers` are available to pleum when they use stdio
(command-based) or HTTP transports. pleum does not support servers using the
deprecated SSE transport.

If a server in `context_servers` has the same name as a pleum extension, pleum
uses its own [configuration](/docs/guides/config-files).
