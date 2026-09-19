# pleum/

pleumcode-specific files kept outside the upstream Goose tree so upstream rebases stay clean.

- `custom_providers/pleum.json` — PleumRouter as a Goose declarative provider (no core changes).
  Install: `cp pleum/custom_providers/pleum.json ~/.config/goose/custom_providers/`, then
  `export PLEUM_API_KEY=...` and `goose run --provider pleum --model gpt-5.4-mini -t "hi"`.
  Model IDs such as `policy/<slug>` / `orch/<slug>` are passed through as-is.
