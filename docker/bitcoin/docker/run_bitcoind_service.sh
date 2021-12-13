#!/usr/bin/env bash

/usr/bin/bitcoind -server -regtest -txindex -zmqpubhashtx=tcp://127.0.0.1:30001 -zmqpubhashblock=tcp://127.0.0.1:30001 -rpcworkqueue=32 -fallbackfee=0.00001000 &
disown
sleep 2
