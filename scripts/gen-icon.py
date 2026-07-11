import struct, zlib

S = 1024

def px(x, y):
    cx, cy = S / 2, S / 2
    d = abs(x - cx) + abs(y - cy)
    if d < 380:
        t = y / S
        return (
            int(56 + (168 - 56) * t),
            int(189 + (85 - 189) * t),
            int(248 + (247 - 248) * t),
            255,
        )
    if d < 402:
        return (223, 230, 255, 255)
    return (0, 0, 0, 0)

rows = b""
for y in range(S):
    rows += b"\x00" + bytes(v for x in range(S) for v in px(x, y))

def chunk(t, d):
    c = t + d
    return struct.pack(">I", len(d)) + c + struct.pack(">I", zlib.crc32(c))

png = (
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", struct.pack(">IIBBBBB", S, S, 8, 6, 0, 0, 0))
    + chunk(b"IDAT", zlib.compress(rows))
    + chunk(b"IEND", b""))
open("app-icon.png", "wb").write(png)
print("wrote app-icon.png")
