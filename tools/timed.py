#!/usr/bin/env python
# Test runner. Naked test commands are banned; run them through this.
#   python scripts/timed.py WU118 C190 -- cargo test -p softedge-core
#   python scripts/timed.py N/A N/A --notes "after the deltas change" -- npm test
# Appends one row to test-time.csv.

import csv
import pathlib
import subprocess
import sys
import time
from datetime import datetime

NOTES_MAX = 200
LOG = pathlib.Path(__file__).resolve().parent.parent / "test-time.csv"
HEADER = ["date", "time", "unit", "lane", "seconds", "exit", "command", "notes"]
USAGE = "usage: timed.py <unit> <lane> [--notes TEXT] -- <command...>"


def main() -> int:
    args = sys.argv[1:]
    if "--" not in args:
        print(USAGE, file=sys.stderr)
        return 2
    cut = args.index("--")
    head, cmd = args[:cut], args[cut + 1 :]
    if not cmd:
        print("no command", file=sys.stderr)
        return 2

    notes = ""
    if "--notes" in head:
        i = head.index("--notes")
        if i + 1 >= len(head):
            print(USAGE, file=sys.stderr)
            return 2
        notes = head[i + 1]
        head = head[:i] + head[i + 2 :]
    if len(head) != 2:
        print(USAGE, file=sys.stderr)
        return 2
    unit, lane = head
    if len(notes) > NOTES_MAX:
        print(f"notes are {len(notes)} chars, max {NOTES_MAX}", file=sys.stderr)
        return 2

    start = time.time()
    try:
        code = subprocess.call(cmd)
    except KeyboardInterrupt:
        code = 130
    secs = round(time.time() - start, 1)

    now = datetime.now()
    new = not LOG.exists()
    with LOG.open("a", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        if new:
            w.writerow(HEADER)
        w.writerow(
            [
                now.date().isoformat(),
                now.strftime("%H:%M"),
                unit,
                lane,
                secs,
                code,
                " ".join(cmd),
                notes,
            ]
        )
    print(f"\n[timed] {secs}s exit={code}", file=sys.stderr)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
