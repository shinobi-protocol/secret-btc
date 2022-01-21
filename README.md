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
acccount a
mnemonic:"orient naive top during tuition orient action health fly snake extra spare robust matter drill broccoli sphere clinic fossil kit hole jungle broccoli cause"
address:secret15sf7vvlyx2zfeq2ank73z9kj4wez6knl4eyc03

acccount b
mnemonic:"person action voice chest push frog insect follow daring ritual dog hamster cream husband pull chair rain clog gauge stereo mask vast during outside"
address:secret1daxrh4yenjvtnhf2qu9rdgpszwzg9794rpck8f
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
