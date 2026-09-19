#!/usr/bin/env python3
"""Rebrand an upstream Goose tree to pleum: file contents and paths.

    python3 pleum/rebrand.py [ROOT]      # default: current directory

Deterministic and idempotent. Run it on a fresh upstream snapshot after each upstream
release (see docs/04-fork-plan.md) instead of hand-editing what it produces, so upstream
merges only ever conflict on real changes.

Deliberately NOT rewritten: `docs/` and `pleum/` (ours), LICENSE/NOTICE (Apache-2.0
attribution must stay), README.md (ours), and the crates.io crate `v8-goose` (a real
external dependency; renaming it breaks the build).
"""
import os
import re
import sys

ROOT = os.path.abspath(sys.argv[1] if len(sys.argv) > 1 else ".")
SKIP_TOP = {".git", "docs", "pleum", "LICENSE", "NOTICE", "README.md"}
SKIP_ANY = {".git", "target", "node_modules", ".claude", ".DS_Store"}

# Order matters: explicit mappings first, so a blind goose->pleum never invents an
# org/repo/domain we don't own (a squattable `aaif-pleum/pleum` would be a supply-chain hole).
RULES = [
    (re.compile(r"(?<![\w-])(?:aaif-goose|block)/goose(?![\w-])"), "gachon-star-want/pleumcode"),
    (re.compile(r"goose-docs\.ai"), "docs.pleum.ai"),
    (re.compile(r"aaif-goose"), "gachon-star-want"),
    (re.compile(r"aaif_goose"), "gachon_star_want"),
    (re.compile(r"(?<!v8[-_])GOOSE"), "PLEUM"),
    (re.compile(r"(?<!v8[-_])Goose"), "Pleum"),
    (re.compile(r"(?<!v8[-_])goose"), "pleum"),
]


def rebrand(s):
    for pat, rep in RULES:
        s = pat.sub(rep, s)
    return s


def files():
    for dirpath, dirnames, filenames in os.walk(ROOT):
        rel = os.path.relpath(dirpath, ROOT)
        dirnames[:] = [d for d in dirnames if d not in SKIP_ANY and not (rel == "." and d in SKIP_TOP)]
        for f in filenames:
            if f in SKIP_ANY or (rel == "." and f in SKIP_TOP):
                continue
            yield os.path.join(dirpath, f)


def main():
    changed = renamed = 0
    moves = []
    for path in list(files()):
        if os.path.islink(path):
            target = os.readlink(path)
            new = rebrand(target)
            if new != target:
                os.unlink(path)
                os.symlink(new, path)
        else:
            with open(path, "rb") as fh:
                raw = fh.read()
            if b"\0" not in raw[:8192]:  # text only; binaries keep their bytes
                text = raw.decode("utf-8", "surrogateescape")
                new = rebrand(text)
                if new != text:
                    with open(path, "wb") as fh:
                        fh.write(new.encode("utf-8", "surrogateescape"))
                    changed += 1
        rel = os.path.relpath(path, ROOT)
        new_rel = rebrand(rel)
        if new_rel != rel:
            moves.append((rel, new_rel))
    for rel, new_rel in moves:
        src, dst = os.path.join(ROOT, rel), os.path.join(ROOT, new_rel)
        if os.path.lexists(dst):
            sys.exit(f"refusing to overwrite existing {new_rel} (from {rel})")
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        os.rename(src, dst)
        renamed += 1
    # drop directories emptied by the moves, walking up from each old location
    for rel, _ in moves:
        d = os.path.dirname(os.path.join(ROOT, rel))
        while d != ROOT and os.path.isdir(d) and not os.listdir(d):
            os.rmdir(d)
            d = os.path.dirname(d)
    print(f"rewrote {changed} files, renamed {renamed} paths")


if __name__ == "__main__":
    main()
