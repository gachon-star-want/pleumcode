# pleum/

pleumcode-specific files kept outside the renamed upstream tree so upstream merges stay clean.

- `rebrand.py` — the deterministic goose→pleum rename. Re-run it on every fresh upstream snapshot
  (procedure in `docs/04-fork-plan.md`); never hand-edit what it produces.
- `custom_providers/pleum.json` — PleumRouter as a declarative provider (no core changes).
  Install: `cp pleum/custom_providers/pleum.json ~/.config/pleum/custom_providers/`, then
  `export PLEUM_API_KEY=...` and `pleum run --provider pleum --model gpt-5.4-mini -t "hi"`.
  Model IDs such as `policy/<slug>` / `orch/<slug>` are passed through as-is.
