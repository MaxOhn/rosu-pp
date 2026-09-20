"""Scan .osu directoy for diverse osu!standard maps.

Prints a summary of candidate maps grouped by difficulty/rate bucket.
Usage: python select_maps.py [max_samples_per_bucket]

Field semantics per this repo's LegacyBeatmapDecoder (osu.Game/Beatmaps/Formats):
- [General]  Mode:<n>          (absent => mode 0)
- [Difficulty] ApproachRate/CircleSize/OverallDifficulty  (absent => legacy map)
- [TimingPoints] ms,bpm,meter,sampleSet,customBank,sampleVol,timingChange,effects
  (index 1 = bpm; negative bpm is the 1/x rate form)
- [HitObjects] type is field index 4 (5 = spinner)
"""
import sys
from collections import Counter, defaultdict
from pathlib import Path

BEATMAPS = Path(r"C:/Users/Administrator/Coding/rosu-pp-verifier/beatmaps")


def parse(path):
    mode = ar = cs = od = None
    bms = []
    spinners = 0
    objects = 0
    section = ""
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
                if len(parts) >= 5 and parts[4].strip() == "5":
                    spinners += 1
                objects += 1
    if mode is None:
        mode = 0  # osu! default when the Mode: line is absent
    return mode, ar, cs, od, objects, spinners, bms


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


def bucket_of(ar, cs, od, rc, spin):
    a = "legacyAR" if ar is None else ("lowAR" if ar < 6 else ("medAR" if ar < 8.5 else "highAR"))
    c = "legacyCS" if cs is None else ("lowCS" if cs < 3 else ("medCS" if cs < 6 else "highCS"))
    d = "legacyOD" if od is None else ("lowOD" if od < 5 else ("medOD" if od < 7.5 else "highOD"))
    s = "spin" if spin >= 3 else "nospin"
    return (a, c, d, rc, s)


def main():
    max_samples = int(sys.argv[1]) if len(sys.argv) > 1 else 3
    results = []
    for p in sorted(BEATMAPS.iterdir()):
        if p.suffix.lower() != ".osu":
            continue
        try:
            m, ar, cs, od, objs, spin, bms = parse(p)
        except Exception:
            continue
        if m != 0 or (objs or 0) < 50:
            continue
        results.append((p.stem, ar, cs, od, objs, spin, bms))
    print(f"TOTAL standard maps with >=50 objects: {len(results)}")
    c = Counter()
    buckets = defaultdict(list)
    for r in results:
        _, ar, cs, od, objs, spin, bms = r
        key = bucket_of(ar, cs, od, rate_bucket(bms), spin)
        c[key] += 1
        buckets[key].append(r)
    for k in sorted(c):
        print(k, c[k])
    print("\nSAMPLES (id ar cs od objs spin bms):")
    for key in sorted(buckets):
        for r in buckets[key][:max_samples]:
            bms = r[6]
            show = bms[:6]
            print(key, "->", r[0], r[1], r[2], r[3], r[4], r[5], show)


if __name__ == "__main__":
    main()
