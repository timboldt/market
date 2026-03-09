#!/bin/bash
# Run multiple simulations with different seeds and collect health metrics.
# Usage: ./run_batch.sh [ticks] [seeds...]
# Example: ./run_batch.sh 2000 42 123 456 789 1000

TICKS="${1:-2000}"
shift
SEEDS="${@:-42 123 456 789 1000}"

BINARY="./target/release/market"

if [ ! -f "$BINARY" ]; then
    echo "Building release binary..."
    cargo build --release
fi

echo "============================================"
echo "Batch run: ${TICKS} ticks per seed"
echo "Seeds: ${SEEDS}"
echo "============================================"
echo ""

for SEED in $SEEDS; do
    echo "────────────────────────────────────────────"
    echo "Seed: ${SEED}"
    echo "────────────────────────────────────────────"

    # Run quietly with graph output in a per-seed subdirectory
    OUTDIR="output/seed_${SEED}"
    mkdir -p "$OUTDIR"

    $BINARY --seed "$SEED" --ticks "$TICKS" --speed 0 --quiet --graph 2>&1

    # Move charts to per-seed directory
    for f in output/prices.png output/trades.png output/roles.png output/wealth.png; do
        [ -f "$f" ] && mv "$f" "$OUTDIR/"
    done

    echo ""
done

echo "============================================"
echo "All runs complete. Charts saved in output/seed_*/"
echo "============================================"
