/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { Account, SecretNetworkClient } from 'secretjs';
import { encodingLength } from 'bip174/src/lib/converter/varint';
import { address, Network, Transaction } from 'bitcoinjs-lib';
import { MerkleProof } from '../bitcoin_spv/BitcoinMerkleTree';
import { MerkleProof as TendermintMerkleProof } from '../sfps/TendermintMerkleTree';
import {
    HandleMsg,
    QueryMsg,
    QueryAnswer,
    Convert,
    QueryAnswerSuspensionSwitch,
} from './types';
import { SFPSClient } from '../sfps/SFPSClient';
import { TokenClient } from '../token/TokenClient';
import {
    ContractClient,
    ExecuteResult as GenericExecuteResult,
} from '../ContractClient';
import BigNumber from 'bignumber.js';
import { Logger } from 'winston';
import { BitcoinSPVClient } from '../bitcoin_spv/BitcoinSPVClient';
import { FinanceAdminClient } from '../finance_admin_v1/FinanceAdminClient';
import { TxResult } from '../../TendermintRPCClient';
import { CurrentHighestHeaderElement as HeaderElement } from '../sfps/types';

type ExecuteResult<ANSWER> = GenericExecuteResult<HandleMsg, ANSWER>;

interface Config {
    btcTxValues: BigNumber[];
    sfpsContractAddress: string;
    sfpsContractHash: string;
    sbtcContractAddress: string;
    sbtcContractHash: string;
    bitcoinSPVContractAddress: string;
    bitcoinSPVContractHash: string;
    financeAdminContractAddress: string;
    financeAdminContractHash: string;
    ownerAddress: string;
}

class GatewayClient extends ContractClient<HandleMsg, QueryMsg, QueryAnswer> {
    public viewingKey?: string;

    private referenceContractClients?: {
        sfps: SFPSClient;
        sbtc: TokenClient;
        bitcoinSPV: BitcoinSPVClient;
        financeAdmin: FinanceAdminClient;
    };

    constructor(
        contractAddress: string,
        secretNetworkClient: SecretNetworkClient,
        logger: Logger,
        viewingKey?: string
    ) {
        super(contractAddress, secretNetworkClient, logger);
        this.viewingKey = viewingKey;
    }

    public async sbtcClient(): Promise<TokenClient> {
        await this.initReferenceContractClients();
        return this.referenceContractClients!.sbtc;
    }

    public async sfpsClient(): Promise<SFPSClient> {
        await this.initReferenceContractClients();
        return this.referenceContractClients!.sfps;
    }

    public async bitcoinSPVClient(): Promise<BitcoinSPVClient> {
        await this.initReferenceContractClients();
        return this.referenceContractClients!.bitcoinSPV;
    }

    public async financeAdminClient(): Promise<FinanceAdminClient> {
        await this.initReferenceContractClients();
        return this.referenceContractClients!.financeAdmin;
    }

    public async verifyMintTx(
        height: number,
        tx: Transaction,
        merkleProof: MerkleProof,
        gasLimit?: number
    ): Promise<ExecuteResult<void>> {
        const msg = {
            verify_mint_tx: {
                height,
                tx: tx.toBuffer().toString('base64'),
                merkle_proof: merkleProof.encodeToContractMsg(),
            },
        };
        return await this.execute(msg, gasLimit || 800000, () => void 0);
    }

    public async releaseIncorrectAmountBTC(
        height: number,
        tx: Transaction,
        merkleProof: MerkleProof,
        recipientAddress: string,
        feePerVb: number,
        gasLimit?: number
    ): Promise<ExecuteResult<Transaction>> {
        const msg = {
            release_incorrect_amount_b_t_c: {
                height,
                tx: tx.toBuffer().toString('base64'),
                merkle_proof: merkleProof.encodeToContractMsg(),
                recipient_address: recipientAddress,
                fee_per_vb: feePerVb,
            },
        };
        return await this.execute(msg, gasLimit || 800000, (answerJson) =>
            Transaction.fromBuffer(
                Buffer.from(
                    Convert.toHandleAnswer(answerJson)
                        .release_incorrect_amount_b_t_c!.tx,
                    'base64'
                )
            )
        );
    }

    public async requestMintAddress(
        entropy: Buffer,
        gasLimit?: number
    ): Promise<ExecuteResult<string>> {
        return await this.execute(
            {
                request_mint_address: {
                    entropy: entropy.toString('base64'),
                },
            },
            gasLimit || 200000,
            (answerJson) =>
                Convert.toHandleAnswer(answerJson).request_mint_address!
                    .mint_address
        );
    }

    // returns RequestKey
    public async requestReleaseBtc(
        amount: BigNumber,
        entropy: Buffer,
        gasLimit?: number
    ): Promise<ExecuteResult<string>> {
        return await this.execute(
            {
                request_release_btc: {
                    amount: amount.shiftedBy(8).toNumber(),
                    entropy: entropy.toString('base64'),
                },
            },
            gasLimit || 600000,
            (answerJson) =>
                Buffer.from(
                    Convert.toHandleAnswer(answerJson).request_release_btc!
                        .request_key
                ).toString('hex')
        );
    }

    // TODO improve gasLimit estimation
    public async claimReleaseBtc(
        headers: HeaderElement[],
        encryption_key: string,
        merkle_proof: TendermintMerkleProof,
        tx_result: TxResult,
        recipient_address: string,
        fee_per_vb: number,
        header_hash_index: number,
        gasLimit?: number
    ): Promise<ExecuteResult<Transaction>> {
        const message = {
            claim_released_btc: {
                encryption_key,
                tx_result_proof: {
                    headers: headers,
                    merkle_proof: {
                        total: merkle_proof.total,
                        index: merkle_proof.index,
                        leaf_hash: merkle_proof.leafHash.toString('hex'),
                        aunts: merkle_proof.aunts.map((aunt) =>
                            aunt.toString('hex')
                        ),
                    },
                    tx_result: {
                        code: tx_result.code,
                        data: tx_result.data,
                        gas_used: tx_result.gas_used,
                        gas_wanted: tx_result.gas_wanted,
                    },
                },
                recipient_address,
                fee_per_vb,
                header_hash_index,
            },
        };
        const result = await this.execute(
            message,
            gasLimit || 5000000,
            (answerJson) =>
                Transaction.fromBuffer(
                    Buffer.from(
                        Convert.toHandleAnswer(answerJson).claim_released_btc!
                            .tx,
                        'base64'
                    )
                )
        );
        this.logger.info('transaction hex:' + result.answer.toHex());
        return result;
    }

    public async setViewingKey(key: string): Promise<ExecuteResult<void>> {
        const result = await this.execute(
            { set_viewing_key: { key: key } },
            110000,
            () => void 0
        );
        this.viewingKey = key;
        return result;
    }

    public async config(): Promise<Config> {
        const result = await this.query({
            config: {},
        });
        const raw = result.config!;
        return {
            btcTxValues: raw.btc_tx_values.map((satoshi) =>
                new BigNumber(satoshi).shiftedBy(-8)
            ),
            sfpsContractAddress: raw.sfps.address,
            sfpsContractHash: raw.sfps.hash,
            sbtcContractAddress: raw.sbtc.address,
            sbtcContractHash: raw.sbtc.hash,
            bitcoinSPVContractAddress: raw.bitcoin_spv.address,
            bitcoinSPVContractHash: raw.bitcoin_spv.hash,
            financeAdminContractAddress: raw.finance_admin.address,
            financeAdminContractHash: raw.finance_admin.hash,
            ownerAddress: raw.owner,
        } as Config;
    }

    public async getMintAddress(
        viewingKey = this.viewingKey
    ): Promise<string | undefined> {
        if (viewingKey === undefined) {
            throw new Error('no viewing key');
        }
        const answer = await this.query({
            mint_address: {
                address: this.secretNetworkClient.address,
                key: viewingKey,
            },
        });
        return answer.mint_address!.address || undefined;
    }

    public async getSuspensionSwitch(): Promise<QueryAnswerSuspensionSwitch> {
        const answer = await this.query({
            suspension_switch: {},
        });
        return answer.suspension_switch!;
    }

    public async releaseBTCByOwner(
        txValue: BigNumber,
        maxInputLength: number,
        recipientAddress: string,
        feePerVb: number,
        gasLimit?: number
    ): Promise<ExecuteResult<Transaction>> {
        const result = await this.execute(
            {
                release_btc_by_owner: {
                    tx_value: txValue.shiftedBy(8).toNumber(),
                    max_input_length: maxInputLength,
                    recipient_address: recipientAddress,
                    fee_per_vb: feePerVb,
                },
            },
            gasLimit || 5000000,
            (answerJson) =>
                Transaction.fromBuffer(
                    Buffer.from(
                        Convert.toHandleAnswer(answerJson).release_btc_by_owner!
                            .tx,
                        'base64'
                    )
                )
        );
        this.logger.info('transaction hex:' + result.answer.toHex());
        return result;
    }

    // https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki
    // https://github.com/bitcoin/bips/blob/master/bip-0144.mediawiki
    // Return Max Weight of Transaction after signed.
    // Weight of Signature in P2WPKH Witness can be 71 or 72.
    // This function calculates Max Weight as the weight of signature to be 72.
    public static calcReleaseTxFee(
        recipientAddress: string,
        network: Network,
        feePerVb: number
    ): number {
        const INPUT_CONSTANT_WEIGHT = 160; // (Transaction Hash(32) + Output Index(4) + Sequence Number(4)) * 4
        const P2WPKH_SCRIPT_SIG_WEIGHT = 4; // (Script Sig Length VarInt(1) + Script Sig(0)) * 4
        const P2WPKH_WITNESS_WEIGHT = 108; // Witness Count VarInt(1) + Signature Length VarInt(1) + Signature (71 or 72) + Pubkey Length Varint (1) + pubkey(33)
        const TX_CONSTANT_WEIGHT = 34; // (Version(4) + Lock Time(4)) * 4 + Marker(1) + Flag(1)
        const OUTPUT_CONSTANT_WEIGHT = 32; // Amount(8) * 4
        const TXIN_COUNT_WEIGHT = 4; // Tx Count VarInt(1) * 4
        const TXOUT_COUNT_WEIGHT = 4; // Tx Count VarInt(1) * 4

        const scriptPubkeyLength = address.toOutputScript(
            recipientAddress,
            network
        ).length;
        const inputWeight =
            INPUT_CONSTANT_WEIGHT +
            P2WPKH_SCRIPT_SIG_WEIGHT +
            P2WPKH_WITNESS_WEIGHT;
        const outputWeight =
            OUTPUT_CONSTANT_WEIGHT +
            (encodingLength(scriptPubkeyLength) + scriptPubkeyLength) * 4;
        return (
            Math.ceil(
                (TX_CONSTANT_WEIGHT +
                    TXIN_COUNT_WEIGHT +
                    inputWeight +
                    TXOUT_COUNT_WEIGHT +
                    outputWeight) /
                4
            ) * feePerVb
        );
    }

    private async initReferenceContractClients() {
        const config = await this.config();
        if (this.referenceContractClients) {
            return;
        }
        this.referenceContractClients = {
            sfps: new SFPSClient(
                config.sfpsContractAddress,
                this.secretNetworkClient,
                this.logger
            ),
            sbtc: new TokenClient(
                config.sbtcContractAddress,
                this.secretNetworkClient,
                this.logger
            ),
            bitcoinSPV: new BitcoinSPVClient(
                config.bitcoinSPVContractAddress,
                this.secretNetworkClient,
                this.logger
            ),
            financeAdmin: new FinanceAdminClient(
                config.financeAdminContractAddress,
                this.secretNetworkClient,
                this.logger
            ),
        };
    }
}

export { GatewayClient, Account, Transaction, Config };
