# shinobi-multisig

Command line interface for shinobi multisig contract.

## Requirement

-   Node.js v16.4.1
-   yarn v1.22.17

## Install

### 1. Clone Repository

```bash
git clone https://github.com/shinobi-protocol/secret-btc
cd secret-btc
```

### 2. Build sbtc-js

Build sbtc-js locally, a library for shinobi protocol.

```bash
cd client/sbtc-js
yarn
yarn build
```

### 3. Build shinobi-multisig and link

Build shinobi-multisig locally.

```bash
cd ../shinobi-multisig
yarn
yarn build
yarn symlink
```

`yarn symlink` symlinks shinobi-multisig binnary files.

### 4. Init Cli

Init cli config.
Following command will create `config.json` to `$HOME/.shinobi-multisig`.

**`config.json` includes mnemonic phrase of your account.**

**Please handle the file with care.**

```bash
shinobi-multisig init
```

## Commands

`shinobi-multisig --help` to show cli help.

```bash
shinobi-multisig

Usage:
  $ shinobi-multisig <command> [options]

Commands:
  init                                               Init cli config.
  show-config                                        Print cli config.
  show-account                                       Print signer account.
  query-multisig-status                              Query multisig contract status (signers, required signs, transaction count).
  query-tx <multisig tx id>                          Query multisig tx.
  submit-tx <contract address> <msg json file path>  Submit new multisig tx. Multisig tx id will be returned.
  sign-tx <multisig tx id>                           Sign a multisig tx. Once the required number of signers have signed the tx, the tx will be executed

For more info, run any command with the `--help` flag:
  $ shinobi-multisig init --help
  $ shinobi-multisig show-config --help
  $ shinobi-multisig show-account --help
  $ shinobi-multisig query-multisig-status --help
  $ shinobi-multisig query-tx --help
  $ shinobi-multisig submit-tx --help
  $ shinobi-multisig sign-tx --help

Options:
  --config-path <path>  File path of config json file. (default: /home/joey/.shinobi-multisig/config.json)
  -h, --help            Display this message
shinobi-multisig

Usage:
  $ shinobi-multisig <command> [options]

Commands:
  init                                               Init cli config.
  show-config                                        Print cli config.
  show-account                                       Print signer account.
  query-multisig-status                              Query multisig contract status (signers, required signs, transaction count).
  query-tx <multisig tx id>                          Query multisig tx.
  submit-tx <contract address> <msg json file path>  Submit new multisig tx. Multisig tx id will be returned.
  sign-tx <multisig tx id>                           Sign a multisig tx. Once the required number of signers have signed the tx, the tx will be executed

For more info, run any command with the `--help` flag:
  $ shinobi-multisig init --help
  $ shinobi-multisig show-config --help
  $ shinobi-multisig show-account --help
  $ shinobi-multisig query-multisig-status --help
  $ shinobi-multisig query-tx --help
  $ shinobi-multisig submit-tx --help
  $ shinobi-multisig sign-tx --help

Options:
  --config-path <path>  File path of config json file. (default: /home/joey/.shinobi-multisig/config.json)
  -h, --help            Display this message
```
