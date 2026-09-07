#!/usr/bin/env bash
#
# End-to-end walkthrough on regtest.
#
# Creates two wallets from two fresh seeds, funds the first by mining, sends a
# payment from the native SegWit wallet to the Taproot one, confirms it, sends
# some back, and verifies both transactions from their raw bytes.
#
# Requires a running node:  ./scripts/regtest-node.sh start
#
#   ./scripts/demo.sh              # run it
#   ./scripts/demo.sh --reset      # delete both wallet databases first

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

WALLET_BIN="./target/release/rfbwallet"
SPENDER_DB=data/wallet.sqlite
RECIPIENT_DB=data/taproot.sqlite
RECIPIENT_SEED=data/taproot.seed

step() { printf '\n\n\033[1m=== %s ===\033[0m\n' "$*"; }
run()  { printf '\n$ %s\n' "$*"; "$@"; }

if [[ "${1:-}" == "--reset" ]]; then
  rm -f "$SPENDER_DB" "$RECIPIENT_DB" "$RECIPIENT_SEED"
  echo "removed existing wallet databases"
fi

[[ -f .env ]] || { echo ".env not found; copy .env.example and set RFB_MNEMONIC" >&2; exit 1; }

step "Build"
cargo build --release --quiet
echo "built $WALLET_BIN"

step "The node"
run $WALLET_BIN node

step "Create the spending wallet (wpkh, BIP84)"
if [[ -f "$SPENDER_DB" ]]; then
  echo "$SPENDER_DB already exists, keeping it"
else
  run $WALLET_BIN init --descriptor wpkh
fi

step "Create the receiving wallet (tr, BIP86) from a second seed"
mkdir -p data
if [[ ! -f "$RECIPIENT_SEED" ]]; then
  $WALLET_BIN new-mnemonic | awk 'NF==12 {print; exit}' > "$RECIPIENT_SEED"
  chmod 600 "$RECIPIENT_SEED"
  echo "generated a second seed into $RECIPIENT_SEED (gitignored)"
fi

recipient() {
  env RFB_WALLET_DB="$RECIPIENT_DB" RFB_MNEMONIC="$(cat "$RECIPIENT_SEED")" "$WALLET_BIN" "$@"
}

if [[ -f "$RECIPIENT_DB" ]]; then
  echo "$RECIPIENT_DB already exists, keeping it"
else
  printf '\n$ RFB_WALLET_DB=%s rfbwallet init --descriptor tr\n' "$RECIPIENT_DB"
  recipient init --descriptor tr
fi

step "Fund the spending wallet"
run $WALLET_BIN mine --blocks 101
run $WALLET_BIN sync
run $WALLET_BIN balance
run $WALLET_BIN utxos

step "Pay 1 BTC from the wpkh wallet to the tr wallet"
DEST=$(recipient address --unused | awk '/^  address /{print $2}')
echo "recipient address: $DEST"
run $WALLET_BIN send --to "$DEST" --amount 100000000 --fee-rate 2
TXID=$($WALLET_BIN txs | awk '$2=="unconfirmed" {print $1; exit}')

step "Confirm it and let both wallets discover it independently"
run $WALLET_BIN mine --blocks 1
run $WALLET_BIN sync
printf '\n$ RFB_WALLET_DB=%s rfbwallet sync\n' "$RECIPIENT_DB"
recipient sync
printf '\n$ RFB_WALLET_DB=%s rfbwallet utxos\n' "$RECIPIENT_DB"
recipient utxos

step "Verify the payment from its raw bytes (rust-bitcoin, no BDK)"
run $WALLET_BIN verify "$TXID"

step "Send some back, spending a Taproot output"
BACK=$($WALLET_BIN address --unused | awk '/^  address /{print $2}')
echo "spender address: $BACK"
printf '\n$ RFB_WALLET_DB=%s rfbwallet send --to %s --amount 40000000 --selection largest-first\n' "$RECIPIENT_DB" "$BACK"
recipient send --to "$BACK" --amount 40000000 --fee-rate 3 --selection largest-first
BACK_TXID=$(recipient txs | awk '$2=="unconfirmed" {print $1; exit}')

run $WALLET_BIN mine --blocks 1
run $WALLET_BIN sync

step "Verify the Taproot key spend"
run $WALLET_BIN verify "$BACK_TXID"

step "Persistence: a second sync has nothing left to do"
run $WALLET_BIN sync
run $WALLET_BIN info

step "The same wallet without its seed"
printf '\n$ RFB_MNEMONIC= rfbwallet info\n'
env RFB_MNEMONIC= $WALLET_BIN info | sed -n '1,10p'

step "Compare the two descriptor types"
run $WALLET_BIN compare

step "Done"
echo "spend txid:  $TXID"
echo "return txid: $BACK_TXID"
