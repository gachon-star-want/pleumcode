# @aaif/pleum-acp-client

TypeScript client library for communicating with Pleum over an existing Agent
Client Protocol (ACP) transport.

This package provides:

- TypeScript types and Zod validators for Pleum ACP extension methods
- `PleumExtClient` for calling Pleum extension methods
- Client capability definitions and MCP Apps helpers

It does not install, resolve, or start the Pleum executable. Applications own
the transport and process lifecycle.

## Installation

```bash
npm install @aaif/pleum-acp-client @agentclientprotocol/sdk
```

## Usage

Compose the Pleum extension client with the standard ACP SDK:

```typescript
import {
  client as createAcpClient,
  methods,
  PROTOCOL_VERSION,
  type Stream,
} from "@agentclientprotocol/sdk";
import { PleumExtClient } from "@aaif/pleum-acp-client";

async function connectToPleum(stream: Stream) {
  const app = createAcpClient({ name: "my-product" });
  const connection = app.connect(stream);
  const pleum = new PleumExtClient(connection.agent);

  await connection.agent.request(methods.agent.initialize, {
    protocolVersion: PROTOCOL_VERSION,
    clientInfo: {
      name: "my-product",
      version: "1.0.0",
    },
    clientCapabilities: {},
  });

  return { connection, pleum };
}
```

The application creates and owns the `stream`, including its connection and
process lifecycle. Call `connection.close()` when the application no longer
needs the connection.

## Development

From `ui/pleum-acp-client`:

```bash
pnpm run build
```

The generated TypeScript types come from the Rust schemas in `crates/pleum`.
