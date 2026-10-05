"""Compare the stored PSD composite with an exported PNG."""

import argparse
import sys

from PIL import Image
from psd_tools import PSDImage
from psd_limits import pixel_limits


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("psd")
    parser.add_argument("png")
    args = parser.parse_args()
    try:
        psd = PSDImage.open(args.psd)
        with pixel_limits(psd):
            merged = psd.topil(apply_icc=False)
        if merged is None:
            raise ValueError("PSD has no stored merged image")
        merged = merged.convert("RGBA")
        with Image.open(args.png) as source:
            exported = source.convert("RGBA")
        if merged.size != exported.size:
            raise ValueError(f"size mismatch: PSD {merged.size}, PNG {exported.size}")
        stored = merged.tobytes()
        png = exported.tobytes()
        count = 0
        max_delta = 0
        first = None
        for offset in range(0, len(stored), 4):
            if stored[offset + 3] == 0 and png[offset + 3] == 0:
                continue
            delta = max(abs(stored[offset + c] - png[offset + c]) for c in range(4))
            max_delta = max(max_delta, delta)
            if delta > 1:
                count += 1
                if first is None:
                    pixel = offset // 4
                    first = (pixel % merged.width, pixel // merged.width)
        print(f"differing pixels: {count}")
        print(f"first differing coordinate: {first}")
        print(f"maximum channel difference: {max_delta}")
        return 1 if count else 0
    except (OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    sys.exit(main())

