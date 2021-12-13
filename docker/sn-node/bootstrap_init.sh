#!/bin/bash

file=~/.secretd/config/genesis.json
if [ ! -e "$file" ]
then
  # init the node
  rm -rf ~/.secretd/*
  rm -rf /opt/secret/.sgx_secrets/*

  if [ -z "${CHAINID}" ]; then
    chain_id="$CHAINID"
  else
    chain_id="supernova-1"
  fi

  mkdir -p ./.sgx_secrets
  secretd config chain-id "$chain_id"
  secretd config output json
  secretd config keyring-backend test

  # export SECRET_NETWORK_CHAIN_ID=supernova-1
  # export SECRET_NETWORK_KEYRING_BACKEND=test
  secretd init banana --chain-id "$chain_id"

  cp ~/node_key.json ~/.secretd/config/node_key.json
  perl -i -pe 's/"stake"/"uscrt"/g' ~/.secretd/config/genesis.json
  perl -i -pe 's/"172800000000000"/"90000000000"/g' ~/.secretd/config/genesis.json # voting period 2 days -> 90 seconds
  perl -i -pe 's/cors_allowed_origins = \[\]/cors_allowed_origins = ["*"]/' ~/.secretd/config/config.toml

  echo "00000000" | secretd keys import a keys/a
  echo "00000000" | secretd keys import b keys/b
  secretd keys add c
  secretd keys list

  secretd add-genesis-account "$(secretd keys show -a a)" 1000000000000000000uscrt
  secretd add-genesis-account "$(secretd keys show -a b)" 1000000000000000000uscrt
  secretd add-genesis-account "$(secretd keys show -a c)" 1000000000000000000uscrt

  secretd gentx a 1000000uscrt --chain-id "$chain_id"
  #secretd gentx b 1000000uscrt --chain-id "$chain_id"

  secretd collect-gentxs
  secretd validate-genesis

  secretd init-bootstrap
  secretd validate-genesis
fi

# run lcp
lcp --proxyUrl http://localhost:1317 --port 1337 --proxyPartial '' &

# sleep infinity
source /opt/sgxsdk/environment && RUST_BACKTRACE=1 secretd start --rpc.laddr tcp://0.0.0.0:26657 --bootstrap