---
title: Offline / Air-gapped Docs
sidebar_position: 95
sidebar_label: Offline Docs
---

# Offline / Air-gapped Docs

The `pleum-doc-guide` skill reads official pleum documentation before answering
pleum-specific questions. By default it reads from `https://docs.pleum.ai`. In
an offline or air-gapped environment, point pleum at a **local copy** instead by
setting `PLEUM_DOCS_ROOT`.

- If `PLEUM_DOCS_ROOT` is set (in `config.yaml` or the environment), pleum uses
  it as the docs root — either a local filesystem path or an HTTP(S) URL.
- If it is not set, pleum falls back to `https://docs.pleum.ai`.

When the root is a local path, pleum reads the docs with its file tools; no
network access is required.

## Docs layout

A docs root contains a docs map and a `docs/` tree:

```
<docs-root>/
├── pleum-docs-map.md
└── docs/
    ├── getting-started/...
    └── guides/...
```

`pleum-docs-map.md` is the index the skill searches first; every page it reads
is referenced by a path listed there.

## Building a local docs root

Build the docs from a pleum checkout using the same version as your pleum
binary, so the docs match the runtime. The standard documentation build already
produces everything pleum needs — a `pleum-docs-map.md` index and a `docs/` tree
of markdown files — so no custom tooling is required:

```bash
git checkout v1.41.0   # match your pleum binary version
cd documentation
npm run build
```

This writes the docs root to `documentation/build/`, containing:

```
build/
├── pleum-docs-map.md
└── docs/
    ├── getting-started/...
    └── guides/...
```

`npm run build` requires registry access, so run it in an online environment.
Then copy the resulting `build/` directory to your air-gapped target location
(for example `/opt/pleum-docs`) and point `PLEUM_DOCS_ROOT` at it.

## Configuring pleum

Set `PLEUM_DOCS_ROOT` in `config.yaml`:

```yaml
PLEUM_DOCS_ROOT: "/opt/pleum-docs"
```

Or via the environment:

```bash
export PLEUM_DOCS_ROOT=/opt/pleum-docs
```

For a managed distribution, bake the docs tree into your image and set
`PLEUM_DOCS_ROOT` in the shipped `config.yaml` or launcher environment.

## Notes

- Documentation links in pleum's answers always render as canonical
  `https://docs.pleum.ai/...` URLs, even when read locally.
- A custom HTTP(S) mirror also works: set `PLEUM_DOCS_ROOT` to its root URL.
- For MCP extension runtime issues offline, see
  [Airgapped/Offline Environment Issues](/docs/troubleshooting/known-issues#airgappedoffline-environment-issues).
