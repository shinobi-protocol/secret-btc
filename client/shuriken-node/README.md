# Shuriken Node

## Shuriken Network and Shuriken Node

To serve non custodial Bitcoin bridge "Shinobi Gateway", our protocol needs block data of Bitcoin blockchain and Secret Network blockchain to be feeded continuously.

Shuriken Network is an incentive network which transports these block data to Shinobi protocol.

Anyone can participate in Shuriken Network by running Shuriken Node and earn SNB token as a reward.

Shuriken Node watches Bitcoin blockchain and Secret Network blockchain generating new blocks, then submits these data to Shinobi protocol's secret contract and receives SNB reward.

## How to join Shuriken Network Public Beta as a Shuriken Node

This document describes how to participate in the [Shuriken Network Public Beta Test](https://medium.com/@ShinobiProtocol/public-beta-test-overview-c56ff9effc06) as Shuriken Node.
Once your Shuriken Node is running, you will receive Test SNB as a reward.

### Prepare SCRT for Shuriken Node Tx Fee

Shuriken Node feeds the protocol blockchain data by executing Secret Contracts with tx fee, so you will need to prepare some SCRT as tx fee.
Create a Secret Network account with some SCRT(10~100 SCRT should be enough for testing) and backup mnemonic seed.

### Download and Build Shuriken Node

Node.js v16.4.1 and yarn are required to build and run node.

1. Clone the repository and build

```
git clone https://github.com/shinobi-protocol/secret-btc
cd secret-btc/client/shuriken-node
yarn
yarn build
cp .env.example .env
```

2. Edit `.env` of shuriken as following

```
MNEMONIC=[YOUR MNEMONIC SEED]
SHURIKEN_ADDRESS=secret1crv605udfmkgjwem233f4eyzs2dutftj0lpr4d
```

### Setup Testnet Bitcoin JSON-RPC API

Shuriken Node requires bitcoin JSON-RPC API served by Bitcoin Core.

1. To setup Bitcoin Core, see <https://github.com/bitcoin/bitcoin/tree/master/doc>.

2. To create login credentials for a JSON-RPC user, see <https://github.com/bitcoin/bitcoin/tree/master/share/rpcauth>.

3. Run bitcoin core on testnet and sync blockchain.

4. Edit `.env` file of shuriken as following

```
BITCOIN_API_TYPE='rpc'
BITCOIN_RPC_USER=[JSON RPC USER]
BITCOIN_RPC_PASSWORD=[JSON RPC PASSWORD]
```

#### Setup SecretNetwork RPC API

Shuriken Node requires SecretNetwork RPC API served by SecretNetwork full node.

To setup SecretNetwork full node, see <https://docs.scrt.network/node-guides/run-full-node-mainnet.html>.

[Figment](https://www.figment.io/) serves hosted rpc api service, DataHub<https://datahub.figment.io/>. You can use this service instead of running full node.

```
SECRET_REST_URL=[Your RPC URL]
```

### Run Node

```
yarn start
```

### Give Us Feedback

Please join our community and share your feedback!

-   [Twitter](https://twitter.com/@ShinobiProtocol)
-   [Telegram](https://t.me/shinobi_protocol)
-   [Discord](https://discord.gg/wm275b6m8d)
