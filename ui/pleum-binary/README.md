# Native Binary Packages for pleum

This directory contains the npm package scaffolding for distributing the
`pleum` Rust binary as platform-specific npm packages.

## Packages

| Package | Platform |
|---------|----------|
| `@aaif/pleum-binary-darwin-arm64` | macOS Apple Silicon |
| `@aaif/pleum-binary-darwin-x64` | macOS Intel |
| `@aaif/pleum-binary-linux-arm64` | Linux ARM64 |
| `@aaif/pleum-binary-linux-x64` | Linux x64 |
| `@aaif/pleum-binary-win32-x64` | Windows x64 |

## Usage

These are platform-specific implementation dependencies and are not intended
to be installed directly. Install `@aaif/pleum-acp` instead. It installs the
appropriate package automatically and provides the `pleum` command. Each
binary package contains its native executable. Its platform-specific internal
command preserves executable permissions during npm packing;
`@aaif/pleum-acp` remains the sole owner of the supported `pleum` command.

## Release preparation

The `.github/workflows/publish-npm.yml` workflow downloads the binaries from an
exact versioned Pleum release and prepares the platform package tarballs.
By default it only uploads the verified tarballs as a workflow artifact. Set
the manual `publish` input to publish them through the protected npm production
environment.
