import json
import os
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

# ── Config ────────────────────────────────────────────────────
REPO      = os.environ['REPO']
GH_TOKEN  = os.environ['GH_TOKEN']
DATA_DIR = Path('.')
API_URL   = f'https://api.github.com/repos/{REPO}/releases'

# ── Fetch releases ────────────────────────────────────────────
req = urllib.request.Request(
    API_URL,
    headers={
        'Authorization': f'Bearer {GH_TOKEN}',
        'Accept': 'application/vnd.github+json',
    }
)
with urllib.request.urlopen(req) as resp:
    raw = json.load(resp)

print(f"Fetched {len(raw)} releases")

# ── Parse ─────────────────────────────────────────────────────
def get_platform(name: str) -> str:
    n = name.lower()
    if 'windows' in n: return 'windows'
    if 'macos'   in n: return 'macos'
    return 'linux'

releases = []
for r in raw:
    assets = [
        {
            'name':      a['name'],
            'platform':  get_platform(a['name']),
            'downloads': a['download_count'],
        }
        for a in r['assets']
    ]
    releases.append({
        'version': r['tag_name'],
        'total':   sum(a['downloads'] for a in assets),
        'assets':  assets,
    })

total    = sum(r['total'] for r in releases)
windows  = sum(a['downloads'] for r in releases for a in r['assets'] if a['platform'] == 'windows')
macos    = sum(a['downloads'] for r in releases for a in r['assets'] if a['platform'] == 'macos')
linux    = sum(a['downloads'] for r in releases for a in r['assets'] if a['platform'] == 'linux')

print(f"Total: {total}  Windows: {windows}  macOS: {macos}  Linux: {linux}")

# ── Sanity check ──────────────────────────────────────────────
count_file = DATA_DIR / 'download_count.txt'
previous   = int(count_file.read_text().strip()) if count_file.exists() else 0

if total < previous:
    raise RuntimeError(f"Total ({total}) is less than previous ({previous}) — aborting")

diff = total - previous
print(f"New downloads since last run: {diff}")

# ── Write files ───────────────────────────────────────────────
today = datetime.now(timezone.utc).strftime('%Y-%m-%d')
now   = datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')

# download_count.txt — single number
count_file.write_text(str(total))

# download_daily.txt — one line per day
daily_file = DATA_DIR / 'download_daily.txt'
daily_line = f"{today} {total} {windows} {macos} {linux} {diff}\n"
with daily_file.open('a') as f:
    f.write(daily_line)
print(f"Appended to download_daily.txt: {daily_line.strip()}")