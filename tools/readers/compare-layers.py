"""Compare softpsd's exported layer pixels with psd-tools' straight RGBA pixels."""

import argparse
from collections import defaultdict, deque
from pathlib import Path
import re
import subprocess
import sys

from psd_tools import PSDImage


ROOT = Path(__file__).resolve().parents[2]
EXPORT = re.compile(r"^(\d+) (.*) (\d+)x(\d+) (.+)$")


def read_exports(output):
    exports = []
    for line in output.splitlines():
        match = EXPORT.fullmatch(line)
        if match:
            number, name, width, height, path = match.groups()
            if int(number) != len(exports) + 1:
                raise ValueError("layer exports are not numbered in file order")
            exports.append((name, int(width), int(height), ROOT / path))
        elif not re.match(r"^\s*(group|layer) '", line):
            raise ValueError(f"unexpected dump output: {line}")
    return exports


def compare_pixels(name, width, height, path, layer):
    if (width, height) != (layer.width, layer.height):
        raise ValueError(
            f"layer {name!r} size mismatch: softpsd {width}x{height}, "
            f"psd-tools {layer.width}x{layer.height}"
        )
    actual = path.read_bytes()
    if len(actual) != width * height * 4:
        raise ValueError(f"layer {name!r} has an invalid RGBA buffer length")
    image = layer.topil(apply_icc=False)
    if image is None:
        if width * height:
            raise ValueError(f"psd-tools has no pixels for layer {name!r}")
        expected = b""
    else:
        if image.size != (width, height):
            raise ValueError(f"psd-tools pixels do not cover layer {name!r}'s rect")
        expected = image.convert("RGBA").tobytes()
    count = 0
    first = None
    for offset in range(0, len(actual), 4):
        if any(abs(actual[offset + c] - expected[offset + c]) > 1 for c in range(4)):
            count += 1
            if first is None:
                pixel = offset // 4
                first = (layer.left + pixel % width, layer.top + pixel // width)
    print(f"layer {name!r} differing pixels: {count}; first differing coordinate: {first}")
    return count == 0


def compare_layers(exports, psd):
    layers = defaultdict(deque)
    for layer in psd.descendants():
        if layer.is_group():
            continue
        if layer.kind != "pixel":
            raise ValueError(f"unsupported psd-tools layer {layer.name!r}: {layer.kind}")
        name = layer.name.replace("\r", " ").replace("\n", " ")
        layers[name].append(layer)
    passed = True
    for name, width, height, path in exports:
        if not layers[name]:
            raise ValueError(f"layer {name!r} is missing from psd-tools")
        if not compare_pixels(name, width, height, path, layers[name].popleft()):
            passed = False
    missing = [name for name, entries in layers.items() if entries]
    if missing:
        raise ValueError(f"layers missing from softpsd: {', '.join(repr(n) for n in missing)}")
    return passed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("psd")
    args = parser.parse_args()
    try:
        psd_path = Path(args.psd).resolve(strict=True)
        dump = subprocess.run(
            ["cargo", "run", "--example", "dump", "--", str(psd_path)],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
        if dump.returncode:
            print(dump.stderr, file=sys.stderr, end="")
            return 1
        exports = read_exports(dump.stdout)
        return 0 if compare_layers(exports, PSDImage.open(psd_path)) else 1
    except (OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
