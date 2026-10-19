#!/bin/sh
# memory of a process by name: dev/mem.sh amane-shell
# PSS is the honest number (shared pages split between the processes using them). GPU numbers come from the DRM fdinfo.
name="${1:-amane-shell}"
for pid in $(pgrep -x "$name"); do
    pss=$(awk '/^Pss:/{s+=$2} END{print s}' /proc/$pid/smaps_rollup 2>/dev/null)
    rss=$(awk '/^VmRSS:/{print $2}' /proc/$pid/status)
    anon=$(awk '/^Anonymous:/{s+=$2} END{print s}' /proc/$pid/smaps_rollup 2>/dev/null)
    threads=$(awk '/^Threads:/{print $2}' /proc/$pid/status)
    gtt=0; vram=0
    for f in /proc/$pid/fdinfo/*; do
        awk '/^drm-memory-gtt:/{g=$2} /^drm-memory-vram:/{v=$2} END{print g+0, v+0}' "$f" 2>/dev/null
    done | sort -u | awk '{g+=$1; v+=$2} END{printf "%d %d", g, v}' > /tmp/.mem-gpu
    read gtt vram < /tmp/.mem-gpu
    echo "$name pid=$pid  PSS=$((pss/1024)) MiB  RSS=$((rss/1024)) MiB  anon=$((anon/1024)) MiB  threads=$threads  gpu(gtt/vram KiB)=${gtt}/${vram}"
done
