---
sidebar_position: 1
title: Use pleum as an ACP agent
sidebar_label: Overview
description: Build clients that connect to pleum over stdio, HTTP, or WebSocket.
---

# Use pleum as an ACP agent

Use the [Agent Client Protocol (ACP)](https://agentclientprotocol.com/) to build
clients that connect to pleum's agent runtime, tools, extensions, and configured
models. These clients can be code editors, desktop, web, or mobile apps,
automated services, or other custom integrations.

For methods provided by pleum in addition to the standard ACP methods, see the
[pleum ACP Reference](/docs/gdk/acp/reference).

## Install

Choose one installation method based on how your ACP client runs pleum.

### Install the pleum CLI

Install the [pleum CLI](/docs/getting-started/installation) to configure a client
to launch `pleum acp` or to run `pleum serve` yourself.

### Install from the ACP Registry

Supported ACP clients can install and manage pleum for you through the
[pleum entry in the ACP Registry](https://agentclientprotocol.com/get-started/registry#pleum).
This does not require a separate pleum CLI installation.

## Run the agent

### Connect over stdio

Configure your ACP client to launch pleum as a subprocess with this command:

```bash
pleum acp
```

The client communicates with pleum through stdin and stdout and manages the
process for the lifetime of the connection.

### Connect over HTTP or WebSocket

Run `pleum serve` when your ACP client connects over HTTP or WebSocket:

```bash
PLEUM_SERVER__SECRET_KEY='a-long-random-secret' pleum serve
```

`PLEUM_SERVER__SECRET_KEY` sets the secret your clients use to authenticate.
When you run `pleum serve` directly, it listens on `127.0.0.1:3284` and exposes
the ACP endpoint at `/acp`. To use a different address, pass `--host` and
`--port`.

#### Authentication

HTTP clients authenticate with the `X-Secret-Key` header. WebSocket clients can
use the same header, but browser-based WebSocket clients must pass the secret in
the `?token=` query parameter. Requests without the correct secret receive a
`401 Unauthorized` response.

:::warning Local development only
Passing `--dangerously-unauthenticated` starts `pleum serve` without
authentication. Use it only when the server is isolated from untrusted traffic.
:::

#### Browser origins

Most clients do not need to configure origins. Browser-based clients served from
a non-loopback origin must allow that origin when starting pleum. To allow both
a local development client and a deployed web client, specify both origins:

```bash
PLEUM_SERVER__SECRET_KEY='a-long-random-secret' pleum serve \
  --allowed-origin 'http://localhost:5173' \
  --allowed-origin 'https://app.example'
```

Specifying any `--allowed-origin` values replaces the default loopback origins,
so include every origin your clients need, including localhost origins used for
development. Origins must match exactly, including the scheme and port.

For remote deployment, TLS, and certificate setup, see
[Running a Remote pleum Server](/docs/guides/remote-pleum-server).
Run `pleum serve --help` for the complete list of options.

## ACP client examples

### Clients over stdio

Browse the [official ACP clients directory](https://agentclientprotocol.com/get-started/clients)
for clients that can run local ACP agents. For a worked example of installing
and configuring pleum as a stdio agent, see the
[Zed setup example](/docs/gdk/acp/zed).

### pleum Desktop over WebSocket

[pleum Desktop](https://github.com/gachon-star-want/pleumcode/tree/main/ui/desktop) is an
ACP client that uses WebSocket. It starts `pleum serve` locally on an available
loopback port and connects to its `/acp` endpoint over WebSocket.
