#!/usr/bin/env bash
#
# Start, stop and inspect a throwaway regtest node for this wallet.
#
# The node runs with -disablewallet on purpose: this project never uses Bitcoin
# Core's wallet RPCs, only the chain and relay ones, and disabling the wallet
# proves it.
#
#   ./scripts/regtest-node.sh start
#   ./scripts/regtest-node.sh status
#   ./scripts/regtest-node.sh cli getblockcount
#   ./scripts/regtest-node.sh stop
#   ./scripts/regtest-node.sh reset   # stop and delete the chain

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATADIR="${RFB_REGTEST_DIR:-$ROOT/.regtest}"
RPC_PORT=18449
P2P_PORT=18448
RPC_USER=rfb
RPC_PASSWORD=rfbregtest

cli() {
  bitcoin-cli -datadir="$DATADIR" -rpcport="$RPC_PORT" \
    -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASSWORD" "$@"
}

write_config() {
  mkdir -p "$DATADIR"
  cat > "$DATADIR/bitcoin.conf" <<CONF
regtest=1
server=1
txindex=1

[regtest]
rpcuser=$RPC_USER
rpcpassword=$RPC_PASSWORD
rpcport=$RPC_PORT
rpcbind=127.0.0.1
rpcallowip=127.0.0.1
bind=127.0.0.1:$P2P_PORT
fallbackfee=0.0002
CONF
}

case "${1:-}" in
  start)
    if cli getblockcount >/dev/null 2>&1; then
      echo "already running at 127.0.0.1:$RPC_PORT (height $(cli getblockcount))"
      exit 0
    fi
    command -v bitcoind >/dev/null || { echo "bitcoind not found on PATH" >&2; exit 1; }
    write_config
    bitcoind -datadir="$DATADIR" -disablewallet -daemon >/dev/null
    for _ in $(seq 1 30); do
      if cli getblockcount >/dev/null 2>&1; then
        echo "regtest node up at 127.0.0.1:$RPC_PORT (datadir $DATADIR)"
        exit 0
      fi
      sleep 1
    done
    echo "node did not come up; see $DATADIR/regtest/debug.log" >&2
    exit 1
    ;;

  stop)
    cli stop >/dev/null 2>&1 && echo "stopping" || echo "not running"
    ;;

  status)
    if cli getblockchaininfo 2>/dev/null; then :; else echo "not running"; fi
    ;;

  reset)
    cli stop >/dev/null 2>&1 || true
    sleep 2
    rm -rf "$DATADIR"
    echo "removed $DATADIR"
    ;;

  cli)
    shift
    cli "$@"
    ;;

  *)
    echo "usage: $0 {start|stop|status|reset|cli <args...>}" >&2
    exit 1
    ;;
esac
