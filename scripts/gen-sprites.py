"""Generate mana familiar sprite sheets. Stdlib only, deterministic.

Sheet layout per character: 3 rows (idle, working, hover)
x 4 frames of 16x16 px -> 64x48 RGBA PNG.
"""
import struct, zlib

SIZE = 16
STATES = ["idle", "working", "hover"]

CLAWD_PALETTE = {
    "D": (160, 75, 46, 255),   # dark rust (dimmed marks)
    "R": (217, 119, 87, 255),  # clay orange body
    "K": (40, 20, 14, 255),    # face/chevron marks
    "W": (255, 248, 239, 255), # sparkle
}

NIMBUS_PALETTE = {
    "N": (27, 42, 85, 255),    # navy outline
    "B": (79, 127, 217, 255),  # blue
    "L": (143, 183, 240, 255), # light blue
    "V": (16, 26, 61, 255),    # visor
    "C": (126, 240, 255, 255), # cyan glyph
    "W": (234, 243, 255, 255), # highlight
    "G": (53, 84, 143, 255),   # shading
}

CLAWD_BASE = [
    "................",
    "................",
    "................",
    "..RRRRRRRRRRRR..",
    "..RRRRRRRRRRRR..",
    "RRRRKKRRRRKKRRRR",
    "RRRRKKRRRRKKRRRR",
    "..RRRRRRRRRRRR..",
    "..RRRRRRRRRRRR..",
    "..RRRRRRRRRRRR..",
    "..RRRRRRRRRRRR..",
    "..RRRRRRRRRRRR..",
    "..RR.RR..RR.RR..",
    "..RR.RR..RR.RR..",
    "..RR.RR..RR.RR..",
    "................",
]

NIMBUS_BASE = [
    "................",
    ".....BBBB.......",
    "...BBLWLLBB.....",
    "..BLLLLLLLLB....",
    ".BLLLLLLLLLLB...",
    ".BLLNNNNNNNNLB..",
    "BLLLNCVVVVVNLLLB",
    "BLLLNVCVCCVNLLLB",
    "BLLLNCVVVVVNLLLB",
    ".BLLNNNNNNNNLB..",
    "..BLLLLLLLLLLB..",
    "....BLLLLLLB....",
    "..BBLCLCCLLBB...",
    "....BLLLLLLB....",
    "....BGB..BGB....",
    "....NGN..NGN....",
]


def g(grid):
    return [list(r) for r in grid]


def s(grid):
    return ["".join(r) for r in grid]


def shift_rows(grid, rows, dx=0, dy=0):
    """Shift the given row indices by dx/dy, clipping at edges."""
    src = g(grid)
    out = g(grid)
    for r in rows:
        out[r] = ["."] * SIZE
    for r in rows:
        nr = r + dy
        if not 0 <= nr < SIZE:
            continue
        for c in range(SIZE):
            nc = c + dx
            if 0 <= nc < SIZE and src[r][c] != ".":
                out[nr][nc] = src[r][c]
    return s(out)


def remap(grid, table, rows=None):
    out = []
    for i, row in enumerate(grid):
        if rows is None or i in rows:
            out.append("".join(table.get(ch, ch) for ch in row))
        else:
            out.append(row)
    return out


def overlay(grid, points, ch):
    out = g(grid)
    for r, c in points:
        if 0 <= r < SIZE and 0 <= c < SIZE:
            out[r][c] = ch
    return s(out)


def clear_rows(grid, rows):
    return [("." * SIZE if i in rows else row) for i, row in enumerate(grid)]


def shift_region(grid, rows, cols, dx=0, dy=0):
    """Shift only the cells in rows x cols by dx/dy, clipping at edges."""
    src = g(grid)
    out = g(grid)
    for r in rows:
        for c in cols:
            out[r][c] = "."
    for r in rows:
        for c in cols:
            ch = src[r][c]
            if ch == ".":
                continue
            nr, nc = r + dy, c + dx
            if 0 <= nr < SIZE and 0 <= nc < SIZE:
                out[nr][nc] = ch
    return s(out)


def clawd_frames():
    base = CLAWD_BASE

    def eyes(rows_map):
        out = list(base)
        for r, row in rows_map.items():
            out[r] = row
        return out

    blink = eyes({5: "RRRRRRRRRRRRRRRR"})
    focus = eyes({
        5: "RRRRKRRRRRRKRRRR",
        6: "RRRRRKRRRRKRRRRR",
        7: "..RRKRRRRRRKRR..",
    })
    happy = eyes({
        5: "RRRRKRRRRRRKRRRR",
        6: "RRRKRKRRRRKRKRRR",
    })

    bob = shift_rows(base, list(range(3, 15)), dy=1)
    idle = [base, base, bob, blink]

    focus_bob = shift_rows(focus, list(range(3, 15)), dy=1)
    working = [focus, focus_bob, focus, focus_bob]

    arms_up = shift_region(happy, [5, 6], [0, 1, 14, 15], dy=-1)
    hop = shift_rows(arms_up, list(range(3, 15)), dy=-1)
    hover = [arms_up, hop, arms_up, happy]

    return [idle, working, hover]


def nimbus_frames():
    base = NIMBUS_BASE
    visor = [6, 7, 8]

    def face(rows_map):
        out = list(base)
        for r, row in rows_map.items():
            out[r] = row
        return out

    squint = face({
        6: "BLLLNCVVVVCNLLLB",
        7: "BLLLNVCVVCVNLLLB",
        8: "BLLLNCVVVVCNLLLB",
    })
    happy = face({
        6: "BLLLNVCVVCVNLLLB",
        7: "BLLLNCVCCVCNLLLB",
        8: "BLLLNVVVVVVNLLLB",
    })

    bob = shift_rows(base, list(range(0, 14)), dy=1)
    blink = remap(base, {"C": "V"}, rows=visor)
    idle = [base, base, bob, blink]

    working = [
        overlay(squint, [(0, 7)], "C"),
        overlay(squint, [(12, 9)], "C"),
        overlay(squint, [(0, 7)], "C"),
        squint,
    ]

    kick_l = shift_rows(happy, [14, 15], dx=-1)
    kick_r = shift_rows(happy, [14, 15], dx=1)
    hover = [kick_l, happy, kick_r, happy]

    return [idle, working, hover]


def build_sheet(frames_by_state, palette):
    w, h = SIZE * 4, SIZE * len(STATES)
    px = [[(0, 0, 0, 0)] * w for _ in range(h)]
    for si, frames in enumerate(frames_by_state):
        for fi, frame in enumerate(frames):
            for r, row in enumerate(frame):
                for c, ch in enumerate(row):
                    if ch != ".":
                        px[si * SIZE + r][fi * SIZE + c] = palette[ch]
    return px


def write_png(path, px):
    h, w = len(px), len(px[0])
    rows = b""
    for row in px:
        rows += b"\x00" + b"".join(bytes(p) for p in row)

    def chunk(t, d):
        c = t + d
        return struct.pack(">I", len(d)) + c + struct.pack(">I", zlib.crc32(c))

    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(rows))
        + chunk(b"IEND", b""))
    with open(path, "wb") as f:
        f.write(png)


def upscale(px, k):
    return [[p for p in row for _ in range(k)] for row in px for _ in range(k)]


def preview(sheets, gap=4, k=8):
    h = max(len(s) for s in sheets) * k
    total_w = sum(len(s[0]) for s in sheets) * k + gap * (len(sheets) - 1)
    out = [[(20, 22, 34, 255)] * total_w for _ in range(h)]
    x = 0
    for sheet in sheets:
        big = upscale(sheet, k)
        for r, row in enumerate(big):
            for c, p in enumerate(row):
                if p[3]:
                    out[r][x + c] = p
        x += len(big[0]) + gap
    return out


if __name__ == "__main__":
    import os
    os.makedirs("public/sprites", exist_ok=True)
    clawd = build_sheet(clawd_frames(), CLAWD_PALETTE)
    nimbus = build_sheet(nimbus_frames(), NIMBUS_PALETTE)
    write_png("public/sprites/clawd.png", clawd)
    write_png("public/sprites/nimbus.png", nimbus)
    write_png("sprites-preview.png", preview([clawd, nimbus]))
    print("wrote public/sprites/{clawd,nimbus}.png and sprites-preview.png")
