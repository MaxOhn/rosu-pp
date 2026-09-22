"""Scan a .osu directory for diverse maps (osu!standard, catch, or mania).

Prints a summary of candidate maps grouped by difficulty/rate bucket.

Usage:
    python select_maps.py <beatmaps_dir> [max_samples_per_bucket] [mode]

Field semantics per this repo's LegacyBeatmapDecoder (osu.Game/Beatmaps/Formats):
- [General]  Mode:<n>          (absent => mode 0), Keys:<n>          (mania key count)
- [Difficulty] ApproachRate/CircleSize/OverallDifficulty  (absent => legacy map)
- [TimingPoints] ms,bpm,meter,sampleSet,customBank,sampleVol,timingChange,effects
  (index 1 = bpm; negative bpm is the 1/x rate form)
- [HitObjects] type is field index 4 (2 = hold, 5 = spinner)
- [Events] Color#<n>          (legacy mania column count, when Keys: is absent)
"""
import sys
from collections import defaultdict
from pathlib import Path


def parse(path):
    """Parse one .osu file. Returns None for anything malformed."""
    mode = ar = cs = od = keys = None
    bms = []
    spinners = holds = objects = 0
    section = ""
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                if line.startswith("["):
                    section = line.strip("[]").lower()
                    continue
                if section == "general":
                    if line.startswith("Mode:"):
                        mode = int(line.split(":", 1)[1].strip())
                    elif line.startswith("Keys:"):
                        keys = int(line.split(":", 1)[1].strip())
                elif section == "difficulty":
                    k = line.split(":", 1)[0].strip().lower()
                    v = line.split(":", 1)[1].strip()
                    if k == "approachrate":
                        ar = float(v)
                    elif k == "circlesize":
                        cs = float(v)
                    elif k == "overalldifficulty":
                        od = float(v)
                elif section == "timingpoints":
                    parts = line.split(",")
                    if len(parts) >= 2:
                        try:
                            bms.append(float(parts[1].strip()))
                        except ValueError:
                            pass
                elif section == "hitobjects":
                    parts = line.split(",")
                    if len(parts) >= 5:
                        t = parts[4].strip()
                        if t == "5":
                            spinners += 1
                        elif t == "2":
                            holds += 1
                    objects += 1
                elif section == "events" and keys is None and line.startswith("Color#"):
                    keys = int(line[len("Color#"):].strip().rstrip("#"))
        if mode is None:
            mode = 0  # osu! default when the Mode: line is absent
        cols = None if cs is None else int(round(cs))  # mania column count comes from CircleSize
    except (OSError, ValueError):
        return None
    return mode, ar, cs, od, objects, spinners, bms, keys, holds, cols


def rate_bucket(bms):
    if not bms:
        return "noTP"
    # round to dodge float noise; sign matters (-bpm is the 1/x rate form)
    distinct = {round(b, 4) for b in bms}
    if len(distinct) > 1:
        return "RC"
    b = next(iter(distinct))
    if abs(b - 100.0) < 1e-9:
        return "const1"
    return "constRate"


def length_bucket(objs):
    return "short" if objs < 500 else ("mid" if objs < 2000 else "long")


def main():
    if len(sys.argv) < 2:
        sys.exit(f"usage: {sys.argv[0]} <beatmaps_dir> [max_samples_per_bucket] [mode]")
    beatmaps = Path(sys.argv[1])
    try:
        max_samples = int(sys.argv[2]) if len(sys.argv) > 2 else 3
        want_mode = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    except (ValueError, IndexError):
        sys.exit(f"usage: {sys.argv[0]} <beatmaps_dir> [max_samples_per_bucket] [mode]")
    results = []
    for p in sorted(beatmaps.iterdir()):
        if p.suffix.lower() != ".osu":
            continue
        r = parse(p)
        if r is None:
            continue
        m, ar, cs, od, objs, spin, bms, keys, holds, cols = r
        if m != want_mode or (objs or 0) < 50:
            continue
        results.append((p.stem, ar, cs, od, objs, spin, bms, keys, holds, cols))
    print(f"TOTAL mode {want_mode} maps with >=50 objects: {len(results)}")
    buckets = defaultdict(list)
    for r in results:
        stem, ar, cs, od, objs, spin, bms, keys, holds, cols = r
        rc = rate_bucket(bms)
        if want_mode == 3:  # mania; column count comes from CircleSize in both decoders
            k = "legacy" if cols is None else f"k{cols}"
            h = "nohold" if holds == 0 else ("hold" if holds < objs // 10 else "manyhold")
            key = (k, h, rc, length_bucket(objs))
        elif want_mode == 2:  # catch
            c = "legacyCS" if cs is None else ("lowCS" if cs < 3 else ("medCS" if cs < 6 else "highCS"))
            key = (c, rc, length_bucket(objs))
        else:  # osu!standard
            a = "legacyAR" if ar is None else ("lowAR" if ar < 6 else ("medAR" if ar < 8.5 else "highAR"))
            c = "legacyCS" if cs is None else ("lowCS" if cs < 3 else ("medCS" if cs < 6 else "highCS"))
            d = "legacyOD" if od is None else ("lowOD" if od < 5 else ("medOD" if od < 7.5 else "highOD"))
            s = "spin" if spin >= 3 else "nospin"
            key = (a, c, d, rc, s)
        buckets[key].append(r)
    for k in sorted(buckets, key=str):
        print(k, len(buckets[k]))
    hdr = "id ar cs od objs spin keys holds bms" if want_mode in (2, 3) else "id ar cs od objs spin bms"
    print(f"\nSAMPLES ({hdr}):")
    for key in sorted(buckets, key=str):
        for r in buckets[key][:max_samples]:
            show = r[6][:6]
            extra = (r[7], r[8]) if want_mode in (2, 3) else ()
            print(key, "->", r[0], r[1], r[2], r[3], r[4], r[5], *extra, show)


if __name__ == "__main__":
    main()
