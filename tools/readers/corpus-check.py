"""Compare corpus trees from softpsd and psd-tools, recording every result."""

from collections import Counter
from pathlib import Path
import subprocess
import sys

from psd_tools import PSDImage


ROOT = Path(__file__).resolve().parents[2]


def name(value):
    return value.replace("\\", "\\\\").replace("'", "\\'").replace("\r", " ").replace("\n", " ")


def tree(psd):
    lines = []

    def walk(layers, indent=""):
        for layer in reversed(list(layers)):
            group = layer.is_group()
            blend = layer.blend_mode.name.replace("_", "")
            if blend == "COLOR":
                blend = "COLORBLEND"
            line = (
                f"{indent}{'group' if group else 'layer'} '{name(layer.name)}'"
                f" opacity={layer.opacity} blend={blend} visible={str(layer.visible).lower()}"
            )
            if not group:
                line += " bounds=" + ",".join(map(str, layer.bbox))
            if layer.has_mask():
                line += " mask=" + ("off" if layer.mask.disabled else "on")
            lines.append(line)
            if group:
                walk(layer, indent + "  ")

    if len(psd) == 0:
        lines.append(
            f"layer 'Background' opacity=255 blend=NORMAL visible=true bounds=0,0,{psd.width},{psd.height}"
        )
    else:
        walk(psd)
    return lines


def first_difference(ours, theirs):
    for i in range(max(len(ours), len(theirs))):
        left = ours[i] if i < len(ours) else "<missing>"
        right = theirs[i] if i < len(theirs) else "<missing>"
        if left != right:
            return f"line {i + 1}: softpsd={left}; psd-tools={right}"
    return None


def main():
    subprocess.run(["cargo", "build", "--example", "dump"], cwd=ROOT, check=True)
    dump = ROOT / "target/debug/examples" / ("dump.exe" if sys.platform == "win32" else "dump")
    files = sorted(p for p in (ROOT / "corpus").rglob("*") if p.suffix.lower() in (".psd", ".psb") and p.is_file())
    counts = Counter()
    with (ROOT / "target/corpus-check.txt").open("w", encoding="utf-8") as report:
        for path in files:
            result = subprocess.run([str(dump), str(path)], cwd=ROOT, capture_output=True, encoding="utf-8", errors="replace")
            try:
                expected = tree(PSDImage.open(path))
                external_error = None
            except Exception as error:
                external_error = str(error)
            if result.returncode:
                category = "refused"
                detail = result.stderr.strip() or f"exit {result.returncode}"
            elif external_error is not None:
                category, detail = "psd-tools failed", external_error
            else:
                actual = [line for line in result.stdout.splitlines() if line.lstrip().startswith(("layer '", "group '"))]
                detail = first_difference(actual, expected)
                category = "differs" if detail else "match"
            counts[category] += 1
            detail = (detail or "").replace("\r", " ").replace("\n", " ").replace("\t", " ")
            report.write(f"{category}\t{path.relative_to(ROOT).as_posix()}\t{detail}\n")
    for category in ("match", "differs", "refused", "psd-tools failed"):
        print(f"{category}: {counts[category]}")
    return 0


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    sys.exit(main())

