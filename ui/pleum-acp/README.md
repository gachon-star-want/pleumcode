# @aaif/pleum-acp

Install and resolve the Pleum executable through npm.

This package distributes the Pleum CLI using platform-specific optional npm
dependencies. It does not contain or depend on the Pleum ACP client.

## Installation

```bash
npm install @aaif/pleum-acp
```

The matching `@aaif/pleum-binary-*` package is installed automatically. Do not
install a platform package directly; `@aaif/pleum-acp` provides the supported
`pleum` command.

## Usage

Run the Pleum CLI installed by the package:

```bash
npx pleum acp
npx pleum serve
```

The launcher forwards arguments and standard input, output, and error streams to
the native executable. It preserves the executable's exit status and forwards
termination signals.

Resolve the executable path programmatically:

```typescript
import { resolvePleumBinary } from "@aaif/pleum-acp";

const binaryPath = resolvePleumBinary();
```

`resolvePleumBinary()` first uses `PLEUM_BINARY` when it is set. Otherwise, it
selects the package matching `process.platform` and `process.arch`. In both
cases it verifies that the executable exists and returns an absolute path.

Use the override to run a locally built or custom Pleum executable:

```bash
PLEUM_BINARY=/path/to/pleum npx pleum acp
```

`PLEUM_BINARY` must point directly to a native Pleum executable, not a
`node_modules/.bin/pleum` command shim.

Supported platforms:

| Operating system | Architecture |
| ---------------- | ------------ |
| macOS            | ARM64        |
| macOS            | x64          |
| Linux            | ARM64        |
| Linux            | x64          |
| Windows          | x64          |

Package managers must install optional dependencies. If optional dependencies
are disabled, the resolver reports which platform package is missing.
