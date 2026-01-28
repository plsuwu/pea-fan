#!/usr/bin/env python

"""
creates a subset version of Iosevka (unnecessarily large by default - like ~140MB zipped) using
only those unicode ranges i THINK i care about...
"""

import shlex
import subprocess
from pathlib import Path
from typing import List

UNICODE_RANGES = [
    "U+0020-007F",  # Basic Latin (ASCII)
    "U+00A0-00FF",  # Latin-1 Supplement
    "U+0030-0039",  # Digits 0-9
    "U+0041-0046",  # Uppercase A-F (hex)
    "U+0061-0066",  # Lowercase a-f (hex)
    "U+0078",  # 'x' for 0x prefix
    "U+2400-243F",  # Control Pictures (␀, ␁, ␂, ..., ␊, ␍, ␛)
    "U+00B6",  # Pilcrow ¶ (paragraph mark)
    "U+21B5",  # ↵ (carriage return symbol)
    "U+2500-257F",  # Box Drawing Characters
    "U+2580-259F",  # Block Elements
    "U+2190-21FF",  # Arrows
    "U+27F0-27FF",  # Supplemental Arrows-A
    "U+2900-297F",  # Supplemental Arrows-B
    "U+2B00-2BFF",  # Miscellaneous Symbols and Arrows
    "U+2200-22FF",  # Mathematical Operators (∀, ∃, ∈, ∑, ⊕)
    "U+2300-23FF",  # Miscellaneous Technical
    "U+2A00-2AFF",  # Supplemental Mathematical Operators
    "U+25A0-25FF",  # Geometric Shapes (■, □, ▲, △)
    "U+2600-26FF",  # Miscellaneous Symbols (☠, ⚠, ⚡)
    "U+2700-27BF",  # Dingbats (✓, ✗, ✱)
    "U+2000-206F",  # General Punctuation
    "U+2E00-2E7F",  # Supplemental Punctuation
    "U+0021",  # ! Exclamation
    "U+003F",  # ? Question
    "U+0023",  # # Hash/Number
    "U+0024",  # $ Dollar
    "U+0025",  # % Percent
    "U+0026",  # & Ampersand
    "U+002A",  # * Asterisk
    "U+002B",  # + Plus
    "U+002D",  # - Minus/Hyphen
    "U+003D",  # = Equals
    "U+005E",  # ^ Caret
    "U+007E",  # ~ Tilde
    "U+0028-0029",  # ( )
    "U+005B-005D",  # [ ]
    "U+007B-007D",  # { }
    "U+003C-003E",  # < >
    "U+00A0",  # Non-breaking space
    "U+2002-2009",  # Various spaces (en space, em space, thin space, etc.)
    "U+0000",  # NULL (if shown)
    "U+FFFD",  # Replacement character �
]


def do_subset(in_path: Path, out_path: Path, unicodes: List[str]):
    cmd_str = f"""pyftsubset {str(in_path)} --output-file={out_path} \
        --unicodes={",".join(unicodes)} --flavor=woff2 \
        --layout-features=* --desubroutinize --no-hinting \
        --drop-tables-= --recalc-bounds --recalc-average-width \
        --name-IDs=* --name-languages=*"""

    cmd = shlex.split(cmd_str)
    # print(cmd)

    try:
        print(f"processing: {in_path.name} -> {out_path.name}")
        subprocess.run(cmd, check=True, capture_output=True)

        original_size = in_path.stat().st_size
        subset_size = out_path.stat().st_size
        change = ((original_size - subset_size) / original_size) * 100

        print(f"reduced: {original_size} -> {subset_size}")
        print(f"\t\t(-{original_size - subset_size}B // -{change:.2f}%)")

    except subprocess.CalledProcessError as e:
        print(f"err: couldn't process '{in_path.name}':")
        print("\t", e.stderr.decode())


if __name__ == "__main__":
    fonts = subprocess.run(["ls", "original"], check=True, capture_output=True)
    fonts = fonts.stdout.decode("utf-8").splitlines()

    for font in fonts:
        path_in = Path(f"./original/{font}")
        path_out = Path(f"./{font}")

        do_subset(path_in, path_out, UNICODE_RANGES)

    # path_in = Path("asdf")
    # path_out = Path("asdfasdf")
    # do_subset(path_in, path_out, UNICODE_RANGES)
