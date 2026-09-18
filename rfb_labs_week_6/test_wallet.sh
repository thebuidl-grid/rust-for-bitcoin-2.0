#!/bin/bash
# Bitcoin Wallet Test Script
# This script tests all wallet functionality

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║      Bitcoin Wallet - Automated Test Suite            ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test
run_test() {
    local test_name=$1
    local command=$2

    echo -n "Testing: $test_name ... "

    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}"
        ((TESTS_FAILED++))
    fi
}

echo "Step 1: Checking Prerequisites"
echo "════════════════════════════════════════════════════════"
echo ""

# Check if Bitcoin Core is running
echo -n "Bitcoin Core running ... "
if bitcoin-cli -regtest getblockchaininfo > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ NOT RUNNING${NC}"
    echo ""
    echo -e "${YELLOW}Please start Bitcoin Core:${NC}"
    echo "  bitcoind -regtest &"
    echo "  OR"
    echo "  docker run -p 18332:18332 -e BITCOIN_RPC_USER=bitcoin -e BITCOIN_RPC_PASSWORD=testpassword kylemanna/bitcoind -regtest"
    exit 1
fi

# Check if .env exists
echo -n ".env file exists ... "
if [ -f ".env" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ MISSING${NC}"
    echo "Creating .env from template..."
    cp .env.example .env
    echo -e "${GREEN}✓ Created${NC}"
fi

echo ""
echo "Step 2: Building Wallet"
echo "════════════════════════════════════════════════════════"
echo ""

run_test "Wallet builds successfully" "cargo build --release > /dev/null 2>&1"

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo ""
echo "Step 3: Testing Wallet Commands"
echo "════════════════════════════════════════════════════════"
echo ""

# Test 1: Initialize wallet
run_test "Initialize wallet" "cargo run --release -- init 2>/dev/null"

# Test 2: Get address
run_test "Get receiving address" "cargo run --release -- address --chain receive 2>/dev/null"

# Test 3: Get change address
run_test "Get change address" "cargo run --release -- address --chain change 2>/dev/null"

# Test 4: Show balance
run_test "Show balance" "cargo run --release -- balance 2>/dev/null"

# Test 5: List UTXOs
run_test "List UTXOs" "cargo run --release -- list-utxos 2>/dev/null"

# Test 6: Show blockchain info
run_test "Get blockchain info" "cargo run --release -- info 2>/dev/null"

# Test 7: Show history
run_test "Show transaction history" "cargo run --release -- history --limit 10 2>/dev/null"

echo ""
echo "Step 4: Testing Fund Generation"
echo "════════════════════════════════════════════════════════"
echo ""

# Generate initial blocks if none exist
BLOCK_COUNT=$(bitcoin-cli -regtest getblockchaininfo | grep '"blocks"' | awk -F: '{print $2}' | tr -d ', ')
echo "Current block height: $BLOCK_COUNT"

if [ "$BLOCK_COUNT" -lt 101 ]; then
    echo "Generating 101 blocks (may take a moment)..."
    run_test "Generate 101 blocks" "bitcoin-cli -regtest generatetoaddress 101 \$(bitcoin-cli -regtest getnewaddress) > /dev/null 2>&1"
else
    echo "Enough blocks already generated ✓"
fi

echo ""
echo "Step 5: Sending Test Bitcoin"
echo "════════════════════════════════════════════════════════"
echo ""

# Get wallet address
echo "Getting wallet address..."
WALLET_ADDR=$(cargo run --release -- address 2>/dev/null | grep '^\[0\]' | awk '{print $NF}')

if [ -z "$WALLET_ADDR" ]; then
    echo -e "${RED}Failed to get wallet address${NC}"
    WALLET_ADDR="Unknown"
else
    echo "Wallet address: $WALLET_ADDR"
fi

# Send bitcoin to wallet
if [ "$WALLET_ADDR" != "Unknown" ]; then
    echo "Sending 5 BTC to wallet..."
    TXID=$(bitcoin-cli -regtest sendtoaddress "$WALLET_ADDR" 5.0)
    echo "Transaction ID: $TXID"

    echo "Confirming transaction (3 blocks)..."
    bitcoin-cli -regtest generatetoaddress 3 $(bitcoin-cli -regtest getnewaddress) > /dev/null 2>&1
    echo -e "${GREEN}✓ Confirmed${NC}"
fi

echo ""
echo "Step 6: Verifying Funds"
echo "════════════════════════════════════════════════════════"
echo ""

echo "Checking wallet balance..."
cargo run --release -- balance 2>/dev/null

echo ""
echo "Listing UTXOs..."
cargo run --release -- list-utxos 2>/dev/null

echo ""
echo "Step 7: Testing Persistence"
echo "════════════════════════════════════════════════════════"
echo ""

BALANCE_BEFORE=$(cargo run --release -- balance 2>/dev/null | grep "Total:" | awk '{print $2}')
echo "Balance before restart: $BALANCE_BEFORE satoshis"

echo "Simulating wallet restart..."
sleep 1

BALANCE_AFTER=$(cargo run --release -- balance 2>/dev/null | grep "Total:" | awk '{print $2}')
echo "Balance after restart: $BALANCE_AFTER satoshis"

if [ "$BALANCE_BEFORE" == "$BALANCE_AFTER" ]; then
    echo -e "${GREEN}✓ Persistence working!${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ Balance changed after restart${NC}"
    ((TESTS_FAILED++))
fi

echo ""
echo "════════════════════════════════════════════════════════"
echo "Test Results"
echo "════════════════════════════════════════════════════════"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              All Tests Passed! ✓                       ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Check the output above.${NC}"
    exit 1
fi
