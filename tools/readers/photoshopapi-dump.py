"""Print a PSD/PSB layer tree and masks through PhotoshopAPI."""

import argparse
import sys

import photoshopapi


def bounds(center_x, center_y, width, height):
    left = round(center_x - width / 2)
    top = round(center_y - height / 2)
    return left, top, left + width, top + height


def walk(layers, indent="  "):
    for layer in reversed(layers):
        group = isinstance(layer, photoshopapi.GroupLayer_8bit)
        line = (
            f"{indent}{'group' if group else 'pixel'} {layer.name!r}"
            f" opacity={round(layer.opacity * 255)}"
            f" blend={layer.blend_mode.name.upper()}"
            f" visible={layer.is_visible} clipped={layer.clipping_mask}"
            f" bbox={bounds(layer.center_x, layer.center_y, layer.width, layer.height)}"
        )
        if layer.has_mask():
            position = layer.mask_position
            rect = bounds(position.x, position.y, layer.mask_width(), layer.mask_height())
            line += (
                f" mask bbox={rect} bg={layer.mask_default_color}"
                f" disabled={layer.mask_disabled}"
            )
        print(line)
        if group:
            walk(layer.layers, indent + "  ")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("file")
    args = parser.parse_args()
    try:
        psd = photoshopapi.LayeredFile_8bit.read(args.file)
        print(f"PhotoshopAPI {psd.width}x{psd.height} depth=8 channels={psd.num_channels}")
        print(f"icc: {len(psd.icc)} bytes")
        walk(psd.layers)
        return 0
    except Exception as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    sys.exit(main())
