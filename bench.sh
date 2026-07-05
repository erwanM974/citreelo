#!/usr/bin/env bash
# Criterion baseline helper.
#
# Usage:
#   ./bench.sh save    <baseline-name>   — run all benches, save baseline, copy to criterion-baselines/
#   ./bench.sh compare <baseline-name>   — restore baseline from criterion-baselines/, then compare

set -e

ACTION="$1"
BASELINE="$2"

if [[ -z "$ACTION" || -z "$BASELINE" ]]; then
    echo "Usage: $0 save|compare <baseline-name>" >&2
    exit 1
fi

BENCHES=(bench_build bench_solve bench_parser)
STORE="criterion-baselines"
TARGET="target/criterion"

case "$ACTION" in
    save)
        for bench in "${BENCHES[@]}"; do
            cargo bench --bench "$bench" -- --save-baseline "$BASELINE"
        done
        mkdir -p "$STORE/$BASELINE"
        cp -r "$TARGET/." "$STORE/$BASELINE/"
        echo "Baseline '$BASELINE' saved to $STORE/$BASELINE/"
        ;;
    compare)
        if [[ ! -d "$STORE/$BASELINE" ]]; then
            echo "No saved baseline '$BASELINE' found in $STORE/." >&2
            exit 1
        fi
        mkdir -p "$TARGET"
        cp -r "$STORE/$BASELINE/." "$TARGET/"
        for bench in "${BENCHES[@]}"; do
            cargo bench --bench "$bench" -- --baseline "$BASELINE"
        done
        ;;
    *)
        echo "Unknown action '$ACTION'. Use save or compare." >&2
        exit 1
        ;;
esac
