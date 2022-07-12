# Secret-BTC - Privacy coin backed by BTC

## Join Shuriken Network Public Beta as a Shuriken Node

See [/client/shuriken-node](./client/shuriken-node)

## Play with it on local with regtest bitcoin network

### 1. Compile the contract to wasm

```bash
  make compile
```

#### Compile On macOS

On macOS, clang by Apple is installed, but it does not support wasm targets.
You can install clang based on LLVM by

```bash
  brew install llvm
```

and use it in compilation as follows:

```bash
 AR=/usr/local/opt/llvm/bin/llvm-ar CC=/usr/local/opt/llvm/bin/clang make compile
```

### 2. Start local development docker-compose

```bash
  make start-local
```

This command runs

1. sn-node: secret network local node
2. bitcoin: bitcoin core on regtest network and a REST API Server which allows local apps to interact with the bitcoin core

The accounts on local SecretNetwork with funds:

```text
a_mnemonic="grant rice replace explain federal release fix clever romance raise often wild taxi quarter soccer fiber love must tape steak together observe swap guitar"
b_mnemonic="jelly shadow frog dirt dragon use armed praise universe win jungle close inmate rain oil canvas beauty pioneer chef soccer icon dizzy thunder meadow"
c_mnemonic="chair love bleak wonder skirt permit say assist aunt credit roast size obtain minute throw sand usual age smart exact enough room shadow charge"
d_mnemonic="word twist toast cloth movie predict advance crumble escape whale sail such angry muffin balcony keen move employ cook valve hurt glimpse breeze brick"
```

### 3. Deploy contracts

`make deploy-local`

this command deploy contracts and exports a deploy report to `deproy_report` folder.

### 4. Run Shuriken node

Shuriken node uploads block information of bitcoin and secret network to contracts.

`make shuriken-node-local`

#### Interact with the regtest bitcoin network

Use `bitcoin-cli` in the docker container

```bash
  docker exec -it sbtc_local_bitcoin /bin/bash

  # In docker container,
  # `b` is alias for `bitcoin-cli -regtest`

  #
  # Commnads
  #

  # Create new wallet or load wallet
  bitcoin-cli -regtest createwallet [wallet]
  bitcoin-cli -regtest loadwallet [wallet]

  # Get a new address
  bitcoin-cli -regtest -rpcwallet=[wallet] getnewaddress

  # Set a label to an address
  bitcoin-cli -regtest -rpcwallet=[wallet] setlabel [address] [label]

  # Get address by the label
  bitcoin-cli -regtest -rpcwallet=[wallet] getaddressesbylabel [address]

  # List transactions with the label
  bitcoin-cli -regtest -rpcwallet=[wallet] listtransactions [address]

  # Mine blocks
  bitcoin-cli -regtest -rpcwallet=[wallet] generatetoaddress [number of blocks] [your address]

  # Send 1 BTC
  bitcoin-cli -regtest -rpcwallet=[wallet] -named sendtoaddress address=[recipient] amount=1 fee_rate=25

```

You can run Bitcoin-Qt on the host machine instead of bitcoin docker image and Regtest Util Server.

```bash
  bitcoin-qt -regtest -connect=127.0.0.1:18445
```

#### SecretNetwork browser wallet

- [Keplr](https://chrome.google.com/webstore/detail/keplr/dmkamcknogkgcdfhhbddcghachkejeap)
