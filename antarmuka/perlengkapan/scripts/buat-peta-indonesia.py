#!/usr/bin/env python3
"""Bangun data peta Indonesia untuk halaman Bank Aset > Sebaran.

Membaca GeoJSON batas negara Natural Earth 1:50m (domain publik, https://www.naturalearthdata.com/)
dan menulis `src/components/peta_indonesia_data.rs`: dua jalur SVG (Indonesia, dan negara tetangga
sebagai latar) dalam proyeksi equirectangular sederhana, 1 derajat = 20 satuan.

Peta ditanam di aplikasi, bukan diambil dari server peta: jaringan pemerintah dan CSP aplikasi tidak
mengizinkan permintaan keluar, dan sebaran satker tidak membutuhkan jalan atau nama jalan.

  curl -o ne50.geojson https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_50m_admin_0_countries.geojson
  python3 scripts/buat-peta-indonesia.py ne50.geojson > src/components/peta_indonesia_data.rs

Hanya pustaka standar Python. Jalankan ulang bila rentang atau tingkat penyederhanaan perlu diubah.
"""
import json
import math
import sys

LON0, LON1, LAT0, LAT1 = 94.0, 142.0, -11.8, 6.8
K = 20.0  # satuan SVG per derajat
W, H = (LON1 - LON0) * K, (LAT1 - LAT0) * K


def proj(lon, lat):
    return ((lon - LON0) * K, (LAT1 - lat) * K)


def rdp(pts, eps):
    """Ramer-Douglas-Peucker, iteratif."""
    if len(pts) < 3:
        return pts
    keep = [False] * len(pts)
    keep[0] = keep[-1] = True
    stack = [(0, len(pts) - 1)]
    while stack:
        a, b = stack.pop()
        ax, ay = pts[a]
        bx, by = pts[b]
        dx, dy = bx - ax, by - ay
        length = math.hypot(dx, dy)
        far, idx = -1.0, -1
        for i in range(a + 1, b):
            px, py = pts[i]
            d = abs(dy * (px - ax) - dx * (py - ay)) / length if length else math.hypot(px - ax, py - ay)
            if d > far:
                far, idx = d, i
        if far > eps:
            keep[idx] = True
            stack.append((a, idx))
            stack.append((idx, b))
    return [p for p, k in zip(pts, keep) if k]


def area(pts):
    n = len(pts)
    return abs(sum(pts[i][0] * pts[(i + 1) % n][1] - pts[(i + 1) % n][0] * pts[i][1] for i in range(n))) / 2


def ring_path(ring, eps, min_area):
    pts = [proj(x, y) for x, y in ring]
    if all(not (0 <= x <= W and 0 <= y <= H) for x, y in pts):
        return ""
    if area(pts) < min_area:
        return ""
    s = rdp(pts, eps)
    if len(s) < 4:
        return ""
    return "M" + "L".join(f"{x:.1f} {y:.1f}" for x, y in s) + "Z"


def main():
    data = json.load(open(sys.argv[1], encoding="utf-8"))
    out = {"IDN": "", "OTHER": ""}
    for f in data["features"]:
        a3 = f["properties"].get("ADM0_A3") or f["properties"].get("ISO_A3")
        g = f["geometry"]
        polys = g["coordinates"] if g["type"] == "MultiPolygon" else [g["coordinates"]]
        key = "IDN" if a3 == "IDN" else "OTHER"
        eps, min_area = (0.12, 0.25) if key == "IDN" else (0.5, 4.0)
        for poly in polys:
            out[key] += ring_path(poly[0], eps, min_area)
    print("//! DIHASILKAN oleh `scripts/buat-peta-indonesia.py`; jangan disunting tangan.")
    print("//!")
    print("//! Sumber: Natural Earth 1:50m Admin 0 - Countries (domain publik), disederhanakan.")
    print()
    print(f"pub const LON_MIN: f64 = {LON0:.1f};")
    print(f"pub const LAT_MAKS: f64 = {LAT1:.1f};")
    print(f"pub const SKALA: f64 = {K:.1f};")
    print(f"pub const LEBAR: f64 = {W:.1f};")
    print(f"pub const TINGGI: f64 = {H:.1f};")
    print()
    print(f'pub const INDONESIA: &str = "{out["IDN"]}";')
    print()
    print(f'pub const TETANGGA: &str = "{out["OTHER"]}";')


main()
