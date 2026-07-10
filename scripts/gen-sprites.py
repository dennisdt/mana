"""Generate mana familiar sprite sheets. Stdlib only, deterministic.

Sheet layout per character: 5 rows (idle, working, hover, carried, low)
x 4 frames of 16x16 px -> 64x80 RGBA PNG.
"""
import struct, zlib

SIZE = 16
STATES = ["idle", "working", "hover", "carried", "low"]

CLAWD_PALETTE = {
    "D": (140, 47, 27, 255),   # dark rust outline
    "R": (217, 119, 87, 255),  # coral shell (anthropic clay)
    "O": (240, 154, 111, 255), # light coral
    "W": (255, 248, 239, 255), # eye white / sparkle
    "K": (51, 21, 15, 255),    # pupil / mouth
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
    "....WW....WW....",
    "....WK....KW....",
    ".....R....R.....",
    "..RR.RRRRRR.RR..",
    ".RROROOOOOORORR.",
    "RRO.ROOOOOOR.ORR",
    "RR..ROOOOOOR..RR",
    "....ROOKKOOR....",
    "....RROOOORR....",
    ".....RRRRRR.....",
    "....R.R..R.R....",
    "...D..D..D..D...",
    "................",
    "................",
]

NIMBUS_BASE = [
    "................",
    ".....BBB.BB.....",
    "...BBLLLBLLB....",
    "..BLLWLLLLLLB...",
    ".BLLLLLLLLLLLB..",
    ".BLLLLLLLLLLLB..",
    "..BNNNNNNNNNB...",
    "..NVCVVVVVVVN...",
    "..NVVCVCCVVVN...",
    "..NVCVVVVVVVN...",
    "..BNNNNNNNNNB...",
    "...BLLLLLLLB....",
    "...BLCLLLCLB....",
    "....BLLLLLB.....",
    "....BGB.BGB.....",
    "....NGN.NGN.....",
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
    claw_rows = list(range(5, 9))
    left, right = list(range(0, 5)), list(range(11, 16))

    blink = remap(base, {"W": "R", "K": "R"}, rows=[2, 3])
    pinch = shift_region(base, claw_rows, left, dy=-1)
    idle = [base, pinch, base, blink]

    up_l = shift_region(base, claw_rows, left, dy=-1)
    up_r = shift_region(base, claw_rows, right, dy=-1)
    working = [
        overlay(up_l, [(1, 8)], "W"),
        up_r,
        overlay(up_l, [(1, 6)], "W"),
        up_r,
    ]

    both_up = shift_region(shift_region(base, claw_rows, left, dy=-2), claw_rows, right, dy=-2)
    hop = shift_rows(both_up, [2, 3, 4], dy=-1)
    hover = [both_up, hop, both_up, base]

    tuck = shift_region(shift_region(base, claw_rows, left, dx=1), claw_rows, right, dx=-1)
    carried = [tuck, shift_rows(base, [12, 13], dx=1), tuck, shift_rows(base, [12, 13], dx=-1)]

    droop = shift_rows(base, [2, 3, 4], dy=1)
    dim = remap(droop, {"O": "R"})
    sleepy = remap(dim, {"W": "R"}, rows=[3])
    low = [dim, dim, sleepy, dim]

    return [idle, working, hover, carried, low]


def nimbus_frames():
    base = NIMBUS_BASE
    visor = list(range(7, 10))
    bob = shift_rows(base, list(range(0, 14)), dy=1)
    blink = remap(base, {"C": "V"}, rows=visor)
    idle = [base, base, bob, blink]

    cursor = [
        base,
        overlay(base, [(8, 10)], "C"),
        overlay(base, [(8, 10), (8, 11)], "C"),
        remap(base, {"C": "V"}, rows=[8]),
    ]
    working = [overlay(f, [(0, 7)], "C") if i % 2 == 0 else f for i, f in enumerate(cursor)]

    kick_l = shift_rows(base, [14, 15], dx=-1)
    kick_r = shift_rows(base, [14, 15], dx=1)
    hover = [kick_l, base, kick_r, base]

    squash = shift_rows(base, [1, 2, 3, 4, 5], dy=1)
    carried = [squash, base, squash, base]

    droop = shift_rows(remap(base, {"C": "G"}), [1, 2, 3], dy=1)
    low = [droop, droop, remap(droop, {"G": "V"}, rows=visor), droop]

    return [idle, working, hover, carried, low]


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
