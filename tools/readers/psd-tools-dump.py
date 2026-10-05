# Prints a PSD's tree, masks and raw layer records through psd-tools.
import sys
from psd_tools import PSDImage

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

p = PSDImage.open(sys.argv[1])
print(f"psd-tools {p.width}x{p.height} mode={p.color_mode.name} depth={p.depth} channels={p.channels}")
icc = p.image_resources.get_data(1039)
print(f"icc: {len(icc) if icc else 0} bytes")


def walk(node, ind="  "):
    for l in node:
        m = l.mask
        line = f"{ind}{l.kind} '{l.name}' opacity={l.opacity} blend={l.blend_mode.name} bbox={l.bbox}"
        if m:
            line += f" mask bbox={m.bbox} bg={m.background_color} disabled={m.disabled}"
        print(line)
        if l.is_group():
            walk(l, ind + "  ")


walk(p)
li = p._record.layer_and_mask_information.layer_info
print(f"layer count field: {li.layer_count}")
for r in li.layer_records:
    chans = [(int(c.id), c.length) for c in r.channel_info]
    blocks = [str(k).replace("Tag.", "") for k in r.tagged_blocks.keys()]
    print(f"  '{r.name}' channels={chans} blocks={blocks}")
print(f"merged compression: {p._record.image_data.compression.name}")
