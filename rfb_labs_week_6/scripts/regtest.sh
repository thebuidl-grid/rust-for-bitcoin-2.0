#!/usr/bin/env bash
# Start / stop a local regtest bitcoind for developing this wallet.
#
#   ./scripts/regtest.sh start   # launch bitcoind, print the cookie path
#   ./scripts/regtest.sh stop    # stop it
#   ./scripts/regtest.sh cli ... # run bitcoin-cli against it
#
# The data directory defaults to ./regtest-data (gitignored). Set BITCOIND /
# BITCOIN_CLI if the binaries are not on PATH (the Bitcoin Core snap exposes
# them as bitcoin-core.daemon / bitcoin-core.cli).

set -euo pipefail

DATADIR="${DATADIR:-$(pwd)/regtest-data}"
BITCOIND="${BITCOIND:-bitcoind}"
BITCOIN_CLI="${BITCOIN_CLI:-bitcoin-cli}"
RPC_PORT="${RPC_PORT:-18443}"

case "${1:-}" in
  start)
    mkdir -p "$DATADIR"
    cat > "$DATADIR/bitcoin.conf" <<EOF
regtest=1
server=1
txindex=1
fallbackfee=0.0002
[regtest]
rpcbind=127.0.0.1
rpcport=$RPC_PORT
rpcallowip=127.0.0.1
EOF
    "$BITCOIND" -datadir="$DATADIR" -daemon
    sleep 3
    echo "regtest node up"
    echo "RPC_URL=127.0.0.1:$RPC_PORT"
    echo "RPC_COOKIE=$DATADIR/regtest/.cookie"
    ;;
  stop)
    "$BITCOIN_CLI" -datadir="$DATADIR" -regtest stop
    ;;
  cli)
    shift
    "$BITCOIN_CLI" -datadir="$DATADIR" -regtest "$@"
    ;;
  *)
    echo "usage: $0 {start|stop|cli ...}" >&2
    exit 1
    ;;
esac
