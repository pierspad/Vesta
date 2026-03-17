import re
import subprocess
from pathlib import Path

out_dir = Path("SERIE_TV")
out_dir.mkdir(exist_ok=True)

video_file = Path("Detour(1945).mp4")
srt_files = list(Path(".").glob("*.srt"))

if not srt_files or not video_file.exists():
    raise FileNotFoundError("File video o SRT mancanti nella directory corrente.")

# Estrazione e conversione dei timestamp dal primo file SRT
ref_srt = srt_files[0].read_text(encoding="utf-8")
times = re.findall(r'(\d{2}:\d{2}:\d{2},\d{3}) --> (\d{2}:\d{2}:\d{2},\d{3})', ref_srt)

def to_sec(t_str):
    h, m, s_ms = t_str.split(':')
    s, ms = s_ms.split(',')
    return int(h) * 3600 + int(m) * 60 + int(s) + int(ms) / 1000

blocks = [(to_sec(start), to_sec(end)) for start, end in times]
total_duration = blocks[-1][1]
targets = [total_duration / 3, 2 * total_duration / 3]

splits = [0.0]
for target in targets:
    for i in range(len(blocks) - 1):
        if blocks[i][1] >= target:
            # Calcola il punto medio nello spazio vuoto tra due sottotitoli
            splits.append((blocks[i][1] + blocks[i+1][0]) / 2)
            break
splits.append(None)

def split_media(in_path, out_path, start, end):
    cmd = ["ffmpeg", "-y", "-ss", str(start), "-i", str(in_path)]
    if end:
        cmd.extend(["-t", str(end - start)])
    if in_path.suffix == ".mp4":
        cmd.extend(["-c", "copy"])
    subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

for i in range(3):
    start, end = splits[i], splits[i+1]
    
    out_video = out_dir / f"{i+1}_{video_file.stem}.mkv"
    split_media(video_file, out_video, start, end)
    
    for srt in srt_files:
        out_srt = out_dir / f"{i+1}_{srt.name}"
        split_media(srt, out_srt, start, end)