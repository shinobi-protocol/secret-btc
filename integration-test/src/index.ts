/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { assert } from 'chai';
import { BigNumber } from 'sbtc-js';
import { LogClient } from 'sbtc-js/build/contracts/log/LogClient';
import { GatewayClient } from 'sbtc-js/build/contracts/gateway/GatewayClient';
import { HandleMsg as GatewayHandleMsg } from 'sbtc-js/build/contracts/gateway/types';
import { ShurikenClient } from 'sbtc-js/build/contracts/shuriken/ShurikenClient';
import { MultisigClient } from 'sbtc-js/build/contracts/multisig/MultisigClient';
import { HandleMsg as MultisigHandleMsg } from 'sbtc-js/build/contracts/multisig/types';
import { networks } from 'sbtc-js/build/contracts/bitcoin_spv/BitcoinSPVClient';
import { EncryptionUtilsImpl, Wallet, SecretNetworkClient } from 'sbtc-js/node_modules/secretjs';
import axios from 'axios';
import RegtestUtilClient from 'shuriken-node/build/RegtestUtilClient';
import BtcSyncClient from 'shuriken-node/build/BtcSyncClient';
import TendermintSyncClient from 'shuriken-node/build/TendermintSyncClient';
import { MerkleTree } from 'sbtc-js/build/contracts/bitcoin_spv/BitcoinMerkleTree';
import { TendermintRPCClient } from 'sbtc-js/build/TendermintRPCClient';
import { MerkleProof } from 'sbtc-js/build/contracts/sfps/TendermintMerkleTree';
import { address, Transaction } from 'bitcoinjs-lib';
import { createLogger, transports, format } from 'winston';
import { FeeReportRenderer, FileWriter } from './FeeReportRenderer';
import { ExecuteResult } from 'sbtc-js/build/contracts/ContractClient';
import { randomBytes } from 'crypto';
import { TokenClient } from 'sbtc-js/build/contracts/token/TokenClient';

class FeeReporter {
    private feeReportRenderer: FeeReportRenderer;
    constructor(
        feeReportRenderer: FeeReportRenderer,
    ) {
        this.feeReportRenderer = feeReportRenderer;
    }
    public async report(executeResult: ExecuteResult<any, any>): Promise<void> {
        this.feeReportRenderer.writeRow(
            executeResult,
        );
    }
}

/**
 * Environment setup
 */
const regtestServerUrl = process.env.REGTEST_SERVER_URL!;
const grpcWebUrl = process.env.GRPC_WEB_URL!;
const chainId = process.env.CHAIN_ID!;
const tendermintRpcUrl = process.env.TENDERMINT_RPC_URL!;
const mnemonic = process.env.MNEMONIC!;
const feeReportFilePath = process.env.FEE_REPORT_FILE_PATH!;
const logAddress = process.env.LOG_ADDRESS!;
const gatewayAddress = process.env.GATEWAY_ADDRESS!;
const snbAddress = process.env.SNB_ADDRESS!;
const treasuryAddress = process.env.TREASURY_ADDRESS!;
const shurikenAddress = process.env.SHURIKEN_ADDRESS!;
const receiveAddress = process.env.RECEIVE_ADDRESS!;

const tendermintClient = new TendermintRPCClient(tendermintRpcUrl);

void (async () => {
    /**
     *  SETUP
     */
    const regtestUtilClient = new RegtestUtilClient();
    // Setup Contract Clients
    const wallet = new Wallet(mnemonic);
    const querier = (await SecretNetworkClient.create({
        grpcWebUrl, chainId
    }
    )).query;
    const encryptionUtils = new EncryptionUtilsImpl(querier.registration, undefined, chainId)
    const secretNetworkClient = await SecretNetworkClient.create({
        grpcWebUrl, chainId, wallet, walletAddress: wallet.address, encryptionUtils
    }
    );


    const logger = createLogger({
        transports: [new transports.Console()],
        format: format.combine(format.simple(), format.timestamp()),
    });
    const gatewayClient = new GatewayClient(
        gatewayAddress,
        secretNetworkClient,
        logger
    );
    const snbClient = new TokenClient(
        snbAddress,
        secretNetworkClient,
        logger
    );
    const shurikenClient = new ShurikenClient(
        shurikenAddress,
        secretNetworkClient,
        logger
    );
    const logClient = new LogClient(logAddress, secretNetworkClient, logger);
    const sbtcClient = await gatewayClient.sbtcClient();
    const sfpsClient = await gatewayClient.sfpsClient();
    const bitcoinSPVClient = await gatewayClient.bitcoinSPVClient();
    const financeAdminClient = await gatewayClient.financeAdminClient();
    const gatewayConfig = await gatewayClient.config();
    const multisigClient = new MultisigClient(
        gatewayConfig.ownerAddress,
        secretNetworkClient,
        logger
    );
    console.log(await financeAdminClient.config());
    console.log(
        await financeAdminClient.mintReward(
            secretNetworkClient.address,
            new BigNumber(1),
            new BigNumber(1000),
            await sbtcClient.unitConverter()
        )
    );

    // Setup Fee Reporter
    const feeReporter = new FeeReporter(
        new FeeReportRenderer(new FileWriter(feeReportFilePath)),
    );

    // Set Viewing Keys
    await feeReporter.report(await sbtcClient.setViewingKey('viewing key'));
    await feeReporter.report(await gatewayClient.setViewingKey('viewing key'));
    await feeReporter.report(await logClient.setViewingKey('viewing key'));

    const initialBalance = await sbtcClient.getBalance();
    const btcTxValue = gatewayConfig.btcTxValues[0];

    // Increase Allowances
    await feeReporter.report(
        await sbtcClient.increaseAllowance(
            gatewayClient.contractAddress,
            await sbtcClient.maxValue()
        )
    );
    await feeReporter.report(
        await snbClient.increaseAllowance(
            treasuryAddress,
            await snbClient.maxValue()
        )
    );

    /**
     * Release Incorrect Amount BTC
     */
    await (async () => {
        const result = await gatewayClient.requestMintAddress(
            Buffer.from(randomBytes(32))
        );
        await feeReporter.report(result);
        const mintAddress = result.answer;
        assert.equal(mintAddress.length, 44);
        /**
         * Release Incorrect Amount BTC -- Mint BTC
         */
        const txValueInSatoshi = btcTxValue.shiftedBy(8).toNumber() - 1;
        const incorrectAmountTx = await regtestUtilClient.faucet(
            mintAddress,
            txValueInSatoshi
        );
        assert.equal(incorrectAmountTx.value, txValueInSatoshi);

        // Mine Bitcoin Blocks
        await axios.post(`${regtestServerUrl}/r/generate?key=satoshi&count=6`);

        // Sync Bitcoin Blocks
        const btcSyncClient = new BtcSyncClient(
            shurikenClient,
            bitcoinSPVClient,
            regtestUtilClient,
            50,
            logger
        );
        await feeReporter.report((await btcSyncClient.syncBitcoinHeaders())[0]);

        /**
         * Release Incorrect Amount BTC -- Release
         */
        const block = await regtestUtilClient.getBlockByTxId(
            incorrectAmountTx.txId
        );
        const blockHeight = await regtestUtilClient.getBlockHeight(
            block.getId()
        );
        const tx = block.transactions!.find((tx) => {
            return tx.getId() === incorrectAmountTx.txId;
        })!;
        const tree = MerkleTree.fromTxs(block.transactions!);
        const mp = tree.merkleProof(tx.getHash());

        const queryVerifyMerkleProofResult =
            await bitcoinSPVClient.verifyMerkleProof(blockHeight, tx, mp);
        assert.isTrue(queryVerifyMerkleProofResult);
        // Execute
        const bitcoinTransaction = await (async (): Promise<Transaction> => {
            const result = await gatewayClient.releaseIncorrectAmountBTC(
                blockHeight,
                tx,
                mp,
                receiveAddress,
                100
            );
            await feeReporter.report(result);
            return result.answer;
        })();
        assert.equal(bitcoinTransaction.outs[0].value, 99988999);
        assert.equal(
            address.fromOutputScript(
                bitcoinTransaction.outs[0].script,
                networks.regtest
            ),
            receiveAddress
        );
        const balance = await sbtcClient.getBalance();
        assert.isTrue(balance.isEqualTo(initialBalance));
    })();

    /**
     * Mint -- Request Mint Address
     */
    await (async () => {
        const result = await gatewayClient.requestMintAddress(
            Buffer.from(randomBytes(32))
        );
        await feeReporter.report(result);
        const mintAddress = result.answer;
        assert.equal(mintAddress.length, 44);

        /**
         * Mint -- Mint BTC
         */
        const txValueInSatoshi = btcTxValue.shiftedBy(8).toNumber();
        const deposittedTx = await regtestUtilClient.faucet(
            mintAddress,
            txValueInSatoshi
        );
        assert.equal(deposittedTx.value, txValueInSatoshi);

        // Mine Bitcoin Blocks
        const deposittedTxId = deposittedTx.txId;
        await axios.post(`${regtestServerUrl}/r/generate?key=satoshi&count=6`);

        // Sync Bitcoin Blocks
        const btcSyncClient = new BtcSyncClient(
            shurikenClient,
            bitcoinSPVClient,
            regtestUtilClient,
            50,
            logger
        );
        await feeReporter.report((await btcSyncClient.syncBitcoinHeaders())[0]);

        /**
         * Mint -- Verify Mint Tx
         */

        // Create MerkleProof
        {
            const block = await regtestUtilClient.getBlockByTxId(
                deposittedTxId
            );
            const blockHeight = await regtestUtilClient.getBlockHeight(
                block.getId()
            );
            const tx = block.transactions!.find((tx) => {
                return tx.getId() === deposittedTxId;
            })!;
            const tree = MerkleTree.fromTxs(block.transactions!);
            const mp = tree.merkleProof(tx.getHash());

            const queryVerifyMerkleProofResult =
                await bitcoinSPVClient.verifyMerkleProof(blockHeight, tx, mp);
            assert.isTrue(queryVerifyMerkleProofResult);
            // Execute
            await feeReporter.report(
                await gatewayClient.verifyMintTx(blockHeight, tx, mp)
            );

            // Assert Balance
            const afterBalance = await sbtcClient.getBalance();
            assert.isTrue(
                afterBalance.minus(initialBalance).isEqualTo(btcTxValue)
            );
        }
    })();

    await (async () => {
        /**
         * Release - Request Release
         */
        // Request Release BTC and Get TxHash of the request tx
        const result = await (async (): Promise<ExecuteResult<any, string>> => {
            const result = await gatewayClient.requestReleaseBtc(
                btcTxValue,
                randomBytes(32)
            );
            await feeReporter.report(result);
            return result;
        })();
        const txHash = result.tx.transactionHash;
        const requestKey = result.answer;

        const txHeight = result.tx.height;

        // Add Event to log contract
        await feeReporter.report(
            await logClient.addReleaseRequestConfirmedEvent(
                txHeight,
                requestKey,
                Math.floor(Date.parse(result.tx.timestamp) / 1000),
                txHash
            )
        );

        /**
         * Release - Wait for Request Confirmed
         */
        // Sync SN Blocks
        const tendermintSyncClient = new TendermintSyncClient(
            shurikenClient,
            sfpsClient,
            tendermintClient,
            10,
            logger
        );
        let syncTendermintHeadersResults: ExecuteResult<any, any>[] = [];
        for (; ;) {
            syncTendermintHeadersResults = syncTendermintHeadersResults.concat(
                await tendermintSyncClient.syncTendermintHeaders()
            );
            const bestHash = await sfpsClient.currentHighestHeaderHash();
            const bestHeight = parseInt(
                (await tendermintClient.getBlockByHash(bestHash)).block.header
                    .height
            );
            if (txHeight < bestHeight) {
                break;
            }
            await new Promise((accept) => setTimeout(accept, 5 * 1000));
        }
        await feeReporter.report(syncTendermintHeadersResults[0]);

        /**
         * Release - Claim Release BTC
         */
        // Get Nonce of the request tx.
        /*
        const nonce = (
            await gatewayClient.secretNetworkClient.getNonceByTxId(txHash)
        )[0]!;
        */
        console.log('messages', JSON.stringify(result.tx.tx.body.messages));
        const decodedTx = (await (import("sbtc-js/node_modules/secretjs/dist/protobuf_stuff/cosmos/tx/v1beta1/tx"))).Tx.decode(result.tx.txBytes);
        const decodedValue = (await (import("sbtc-js/node_modules/secretjs/dist/protobuf_stuff/secret/compute/v1beta1/msg"))).MsgExecuteContract.decode(decodedTx.body!.messages[0].value);
        console.log('decodedTx messages', JSON.stringify(decodedTx.body!.messages));
        console.log('decoded message value', JSON.stringify(decodedValue));
        const nonce = decodedValue.msg.slice(0, 32);

        // Restore TxEncryptionKey of the request tx from nonce.
        const txEncryptionKey =
            await encryptionUtils.getTxEncryptionKey(
                nonce
            );

        // Create Merkle Proof
        const txs = await tendermintClient.getTxsInBlock(txHeight);
        const index = txs.findIndex((tx) => tx.hash === txHash);
        const merkleProof = MerkleProof.fromRpcTxs(txs, index);

        // Get Header Chains until the synced light block
        const bestHash = await sfpsClient.currentHighestHeaderHash();
        const bestHeight = parseInt(
            (await tendermintClient.getBlockByHash(bestHash)).block.header
                .height
        );
        const headers = [];
        for (let i = txHeight + 1; i <= bestHeight; i++) {
            const header = (await tendermintClient.getBlock(i)).block.header;
            headers.push(header);
        }

        // Get HeaderHashIndex of the synced light block
        const headerHashIndex = (await sfpsClient.hashListLength()) - 1;
        const feePerVb = 100;

        // Execute and get Signed Bitcoin Transaction
        const bitcoinTransaction = await (async (): Promise<Transaction> => {
            const result = await gatewayClient.claimReleaseBtc(
                headers,
                Buffer.from(txEncryptionKey).toString('base64'),
                merkleProof,
                txs[index].tx_result,
                receiveAddress,
                feePerVb,
                headerHashIndex
            );
            await feeReporter.report(result);
            return result.answer;
        })();

        // Assert
        assert.equal(bitcoinTransaction.outs[0].value, 99989000);
        assert.equal(bitcoinTransaction.virtualSize() * feePerVb, 11000);
        assert.equal(
            GatewayClient.calcReleaseTxFee(
                receiveAddress,
                networks.regtest,
                feePerVb
            ),
            11000
        );
        assert.equal(
            address.fromOutputScript(
                bitcoinTransaction.outs[0].script,
                networks.regtest
            ),
            receiveAddress
        );
        const balance = await sbtcClient.getBalance();
        assert.isTrue(balance.isEqualTo(initialBalance));
        const allowance = await sbtcClient.getAllowance(
            gatewayClient.contractAddress
        );
        assert.isTrue(
            (await sbtcClient.maxValue()).minus(btcTxValue).isEqualTo(allowance)
        );
        assert.isTrue(balance.isEqualTo(initialBalance));
    })();

    /// suspend gateway
    const suspensionSwitch = {
        claim_release_btc: true,
        release_incorrect_amount_btc: true,
        request_mint_address: true,
        request_release_btc: true,
        verify_mint_tx: true,
    };
    const gatewayHandleMsg: GatewayHandleMsg = {
        set_suspension_switch: {
            suspension_switch: suspensionSwitch,
        },
    };
    await feeReporter.report(
        await multisigClient.submitTransaction(
            gatewayHandleMsg,
            await gatewayClient.codeHash,
            gatewayAddress,
            []
        )
    );
    assert.deepEqual(
        await gatewayClient.getSuspensionSwitch(),
        suspensionSwitch
    );
    const multisigHandleMsg: MultisigHandleMsg = {
        change_config: {
            config: {
                required: 1,
                signers: [secretNetworkClient.address, gatewayAddress],
            },
        },
    };
    await feeReporter.report(
        await multisigClient.submitTransaction(
            multisigHandleMsg,
            await multisigClient.codeHash,
            multisigClient.contractAddress,
            []
        )
    );
    assert.deepEqual(await multisigClient.multisigStatus(), {
        config: {
            required: 1,
            signers: [secretNetworkClient.address, gatewayAddress],
        },
        transaction_count: 2,
    });
    const logs = await logClient.queryLog(0, 50);
    console.log(logs);
})();
