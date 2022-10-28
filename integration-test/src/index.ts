/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { assert } from 'chai';
import * as datefns from 'date-fns';
import { BigNumber } from 'sbtc-js';
import { LogClient } from 'sbtc-js/build/contracts/log/LogClient';
import { GatewayClient } from 'sbtc-js/build/contracts/gateway/GatewayClient';
import { HandleMsg as GatewayHandleMsg } from 'sbtc-js/build/contracts/gateway/types';
import { ShurikenClient } from 'sbtc-js/build/contracts/shuriken/ShurikenClient';
import { MultisigClient } from 'sbtc-js/build/contracts/multisig/MultisigClient';
import VestingClient from 'sbtc-js/build/contracts/vesting/VestingClient';
import { HandleMsg as MultisigHandleMsg } from 'sbtc-js/build/contracts/multisig/types';
import { networks } from 'sbtc-js/build/contracts/bitcoin_spv/BitcoinSPVClient';
import { Wallet } from 'sbtc-js/node_modules/secretjs';
import RegtestUtilClient from 'shuriken-node/build/RegtestUtilClient';
import BtcSyncClient from 'shuriken-node/build/BtcSyncClient';
import SFPSSyncClient from 'shuriken-node/build/SFPSSyncClient';
import { MerkleTree } from 'sbtc-js/build/contracts/bitcoin_spv/BitcoinMerkleTree';
import { address, Transaction } from 'bitcoinjs-lib';
import { createLogger, transports, format } from 'winston';
import { FeeReportRenderer, FileWriter } from './FeeReportRenderer';
import { ExecuteResult } from 'sbtc-js/build/contracts/ContractClient';
import { randomBytes } from 'crypto';
import ShinobiClient from 'sbtc-js/build/ShinobiClient';
import { TokenClient } from 'sbtc-js/build/contracts/token/TokenClient';

class FeeReporter {
    executeResults: ExecuteResult<any, any>[] = [];
    private feeReportRenderer: FeeReportRenderer;
    constructor(feeReportRenderer: FeeReportRenderer) {
        this.feeReportRenderer = feeReportRenderer;
    }
    public async report(executeResult: ExecuteResult<any, any>): Promise<void> {
        this.executeResults.push(executeResult);
        this.feeReportRenderer.writeRow(executeResult);
    }
}

/**
 * Environment setup
 */
const regtestServerUrl = process.env.REGTEST_SERVER_URL!;
const grpcWebUrl = process.env.GRPC_WEB_URL!;
const chainId = process.env.CHAIN_ID!;
const mnemonic = process.env.MNEMONIC!;
const feeReportFilePath = process.env.FEE_REPORT_FILE_PATH!;
const logAddress = process.env.LOG_ADDRESS!;
const gatewayAddress = process.env.GATEWAY_ADDRESS!;
const snbAddress = process.env.SNB_ADDRESS!;
const shurikenAddress = process.env.SHURIKEN_ADDRESS!;
const vestingAddress = process.env.VESTING_ADDRESS!;
const receiveAddress = process.env.RECEIVE_ADDRESS!;

void (async () => {
    /**
     *  SETUP
     */
    const regtestUtilClient = new RegtestUtilClient(regtestServerUrl);
    // Setup Contract Clients
    const wallet = new Wallet(mnemonic);
    const shinobiClient = await ShinobiClient.create(
        grpcWebUrl,
        chainId,
        wallet,
        wallet.address
    );

    const logger = createLogger({
        transports: [new transports.Console()],
        format: format.combine(format.simple(), format.timestamp()),
    });
    const gatewayClient = new GatewayClient(
        gatewayAddress,
        shinobiClient,
        logger
    );
    const shurikenClient = new ShurikenClient(
        shurikenAddress,
        shinobiClient,
        logger
    );
    const logClient = new LogClient(logAddress, shinobiClient, logger);
    const sbtcClient = await gatewayClient.sbtcClient();
    const snbClient = new TokenClient(snbAddress, shinobiClient, logger);
    const sfpsClient = await gatewayClient.sfpsClient();
    const bitcoinSPVClient = await gatewayClient.bitcoinSPVClient();
    const gatewayConfig = await gatewayClient.config();
    const multisigClient = new MultisigClient(
        gatewayConfig.ownerAddress,
        shinobiClient,
        logger
    );
    const vestingClient = new VestingClient(
        vestingAddress,
        shinobiClient,
        logger
    );

    // Setup Fee Reporter
    const feeReporter = new FeeReporter(
        new FeeReportRenderer(new FileWriter(feeReportFilePath))
    );

    // Set Viewing Keys
    await feeReporter.report(await sbtcClient.setViewingKey('viewing key'));
    await feeReporter.report(await snbClient.setViewingKey('viewing key'));
    await feeReporter.report(await gatewayClient.setViewingKey('viewing key'));
    await feeReporter.report(await logClient.setViewingKey('viewing key'));

    const initialBalance = await sbtcClient.getBalance();
    const btcTxValue = gatewayConfig.btcTxValues[0];

    // Vesting Tokens
    const lockAmount = new BigNumber(1000);
    const unitConverter = await snbClient.unitConverter();
    const endTime = datefns.add(new Date(), { hours: 1 });
    await feeReporter.report(
        await vestingClient.lock(
            snbClient,
            lockAmount,
            endTime,
            shinobiClient.sn.address
        )
    );

    const latestID = await vestingClient.latestID();
    assert.equal(latestID, 0);
    console.log(
        feeReporter.executeResults[feeReporter.executeResults.length - 1].tx
            .timestamp
    );
    const startTime = datefns.parseISO(
        feeReporter.executeResults[feeReporter.executeResults.length - 1].tx
            .timestamp
    );
    assert.deepEqual((await vestingClient.vestingInfos([latestID]))[0], {
        id: latestID,
        token: {
            address: snbClient.contractAddress,
            hash: await snbClient.codeHash,
        },
        locker: shinobiClient.sn.address,
        recipient: shinobiClient.sn.address,
        start_time: datefns.getUnixTime(startTime),
        end_time: datefns.getUnixTime(endTime),
        locked_amount: unitConverter.unitToContractValue(lockAmount),
        claimed_amount: '0',
        remaining_amount: unitConverter.unitToContractValue(lockAmount),
    });
    assert.deepEqual(
        await vestingClient.vestingSummary(snbClient.contractAddress),
        {
            total_claimed: '0',
            total_locked: unitConverter.unitToContractValue(lockAmount),
            total_remaining: unitConverter.unitToContractValue(lockAmount),
        }
    );

    const beforeClaim = await snbClient.getBalance();
    await feeReporter.report(await vestingClient.claim(latestID));
    const afterClaim = await snbClient.getBalance();

    const claimedAmount = new BigNumber(
        datefns.differenceInSeconds(
            datefns.parseISO(
                feeReporter.executeResults[
                    feeReporter.executeResults.length - 1
                ].tx.timestamp
            ),
            startTime
        )
    )
        .shiftedBy(8)
        .div(new BigNumber(datefns.differenceInSeconds(endTime, startTime)))
        .integerValue(BigNumber.ROUND_DOWN)
        .multipliedBy(1000)
        .shiftedBy(8)
        .shiftedBy(-8)
        .integerValue()
        .shiftedBy(-8);
    console.log('claimed', claimedAmount.toString());
    console.log('beforeClaim', beforeClaim.toString());
    console.log('afterClaim', afterClaim.toString());

    assert.deepEqual(afterClaim.minus(beforeClaim), claimedAmount);
    const remainingAmount = lockAmount.minus(claimedAmount);
    assert.deepEqual((await vestingClient.vestingInfos([latestID]))[0], {
        id: latestID,
        token: {
            address: snbClient.contractAddress,
            hash: await snbClient.codeHash,
        },
        locker: shinobiClient.sn.address,
        recipient: shinobiClient.sn.address,
        start_time: datefns.getUnixTime(startTime),
        end_time: datefns.getUnixTime(endTime),
        locked_amount: unitConverter.unitToContractValue(lockAmount),
        claimed_amount: unitConverter.unitToContractValue(claimedAmount),
        remaining_amount: unitConverter.unitToContractValue(remainingAmount),
    });
    assert.deepEqual(
        await vestingClient.vestingSummary(snbClient.contractAddress),
        {
            total_claimed: unitConverter.unitToContractValue(claimedAmount),
            total_locked: unitConverter.unitToContractValue(lockAmount),
            total_remaining: unitConverter.unitToContractValue(remainingAmount),
        }
    );

    // Increase Allowances
    await feeReporter.report(
        await sbtcClient.increaseAllowance(
            gatewayClient.contractAddress,
            await sbtcClient.maxValue()
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
        await regtestUtilClient.mine(6);

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
        await regtestUtilClient.broadcast(bitcoinTransaction.toHex());
        await regtestUtilClient.mine(1);
        console.log(await regtestUtilClient.fetch(bitcoinTransaction.getId()));
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
        await regtestUtilClient.mine(6);

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
        const sfpsSyncClient = new SFPSSyncClient(
            shurikenClient,
            sfpsClient,
            1,
            logger
        );
        let syncSFPSHeadersResults: ExecuteResult<any, any>[] = [];
        for (;;) {
            syncSFPSHeadersResults = syncSFPSHeadersResults.concat(
                await sfpsSyncClient.syncSFPSHeaders()
            );
            const bestHeight = await sfpsClient.currentHighestHeaderHeight();
            if (txHeight < bestHeight) {
                break;
            }
            await new Promise((accept) => setTimeout(accept, 5 * 1000));
        }
        await feeReporter.report(syncSFPSHeadersResults[0]);

        /**
         * Release - Claim Release BTC
         */
        const proof = await sfpsClient.getResponseDeliverTxProof(txHash);
        const feePerVb = 100;

        // Execute and get Signed Bitcoin Transaction
        const bitcoinTransaction = await (async (): Promise<Transaction> => {
            const result = await gatewayClient.claimReleaseBtc(
                proof,
                receiveAddress,
                feePerVb
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
                signers: [shinobiClient.sn.address, gatewayAddress],
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
            signers: [shinobiClient.sn.address, gatewayAddress],
        },
        transaction_count: 2,
    });
    const logs = await logClient.queryLog(0, 50);
    console.log(logs);
})();
