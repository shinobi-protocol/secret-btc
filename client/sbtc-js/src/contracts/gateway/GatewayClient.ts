/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import { Account, SigningCosmWasmClient } from 'secretjs';
import { Transaction } from 'bitcoinjs-lib';
import { MerkleProof } from '../bitcoin_spv/BitcoinMerkleTree';
import { MerkleProof as TendermintMerkleProof } from '../sfps/TendermintMerkleTree';
import { HandleMsg, QueryMsg, QueryAnswer, Convert } from './types';
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
import { HeaderElement } from '../sfps/types';

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
        signingCosmWasmClient: SigningCosmWasmClient,
        logger: Logger,
        viewingKey?: string
    ) {
        super(contractAddress, signingCosmWasmClient, logger);
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
                requester: this.signingCosmWasmClient.senderAddress,
                fee_per_vb,
                header_hash_index,
            },
        };
        return await this.execute(message, gasLimit || 5000000, (answerJson) =>
            Transaction.fromBuffer(
                Buffer.from(
                    Convert.toHandleAnswer(answerJson).claim_released_btc!.tx,
                    'base64'
                )
            )
        );
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
                address: this.signingCosmWasmClient.senderAddress,
                key: viewingKey,
            },
        });
        return answer.mint_address!.address || undefined;
    }

    private async initReferenceContractClients() {
        const config = await this.config();
        if (this.referenceContractClients) {
            return;
        }
        this.referenceContractClients = {
            sfps: new SFPSClient(
                config.sfpsContractAddress,
                this.signingCosmWasmClient,
                this.logger
            ),
            sbtc: new TokenClient(
                config.sbtcContractAddress,
                this.signingCosmWasmClient,
                this.logger
            ),
            bitcoinSPV: new BitcoinSPVClient(
                config.bitcoinSPVContractAddress,
                this.signingCosmWasmClient,
                this.logger
            ),
            financeAdmin: new FinanceAdminClient(
                config.financeAdminContractAddress,
                this.signingCosmWasmClient,
                this.logger
            ),
        };
    }
}

export { GatewayClient, Account, Transaction, Config };
