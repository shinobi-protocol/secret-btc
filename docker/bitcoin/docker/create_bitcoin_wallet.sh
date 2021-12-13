#!/usr/bin/env bash

# this script was added by shinobi

# create wallet and get BTC
echo 'create wallet "node"'
bitcoin-cli -regtest createwallet node

ADDRESS=$(bitcoin-cli -regtest getnewaddress)
echo "Generate BTC to $ADDRESS"
bitcoin-cli -regtest generatetoaddress 101 $ADDRESS
