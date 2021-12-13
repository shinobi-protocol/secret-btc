/* eslint-disable @typescript-eslint/ban-types */

import { ContractDetails, SigningCosmWasmClient } from 'secretjs';
import { StdFee } from 'secretjs/types/types';
import { Logger } from 'winston';

export interface ExecuteResult<HANDLE_MSG, ANSWER> {
    contractDetails: ContractDetails;
    msg: HANDLE_MSG;
    transactionHash: string;
    answer: ANSWER;
}

interface PaddingMsg {
    padding?: null | string;
}

const REQUEST_MSG_BLOCK_SIZE = 256;
const DEFAULT_GAS_PRICE = 0.25;

export abstract class ContractClient<
    HANDLE_MSG extends Record<string, PaddingMsg | any>,
    QUERY_MSG extends object,
    QUERY_ANSWER
> {
    public contractAddress: string;
    public signingCosmWasmClient: SigningCosmWasmClient;
    public contractDetails: Promise<ContractDetails>;
    public gasPrice: number;
    protected logger: Logger;

    constructor(
        contractAddress: string,
        signingCosmWasmClient: SigningCosmWasmClient,
        logger: Logger,
        gasPrice = DEFAULT_GAS_PRICE
    ) {
        this.contractAddress = contractAddress;
        this.signingCosmWasmClient = signingCosmWasmClient;
        this.contractDetails = this.signingCosmWasmClient.getContract(
            this.contractAddress
        );
        this.gasPrice = gasPrice;
        this.logger = logger;
    }

    public senderAddress(): string {
        return this.signingCosmWasmClient.senderAddress;
    }

    protected async execute<ANSWER>(
        handleMsg: HANDLE_MSG,
        gasLimit: number,
        parseAnswer: (answerJson: string) => ANSWER
    ): Promise<ExecuteResult<HANDLE_MSG, ANSWER>> {
        const paddedMsg = this.padMsg(handleMsg);
        const stdFee = this.stdFee(gasLimit);
        this.logger.log({
            level: 'info',
            message:
                'Execute Start: ' +
                JSON.stringify(paddedMsg) +
                '\nFee: ' +
                JSON.stringify(stdFee),
        });
        const result = await this.signingCosmWasmClient.execute(
            this.contractAddress,
            paddedMsg,
            '',
            [],
            stdFee
        );
        const answerJson = new TextDecoder().decode(result.data as Uint8Array);
        const executeResult = {
            contractDetails: await this.contractDetails,
            msg: handleMsg,
            transactionHash: result.transactionHash,
            answer: parseAnswer(answerJson),
        };
        this.logger.log({
            level: 'info',
            message: 'Execute Succeed: ' + JSON.stringify(executeResult),
        });
        return executeResult;
    }

    protected async query(queryMsg: QUERY_MSG): Promise<QUERY_ANSWER> {
        const paddedMsg = this.padMsg(queryMsg);
        this.logger.log({
            level: 'info',
            message: 'Query Start: ' + JSON.stringify(paddedMsg),
        });
        const result = (await this.signingCosmWasmClient.queryContractSmart(
            this.contractAddress,
            paddedMsg
        )) as QUERY_ANSWER;
        this.logger.log({
            level: 'info',
            message: 'Query Succeed: ' + JSON.stringify(result),
        });
        return result;
    }

    /**
     * Add space to HandleMsg in order to avoid data leakage attacks
     * by analyzing differences in handle msg sizes
     * (https://build.scrt.network/dev/privacy-model-of-secret-contracts.html#outputs-2).
     *
     * The size of padded msg in UTF-8 encoding X satisfies the following relationship.
     *            (256n + 243) <= X <= 256(n+1) (n>=0).
     *
     * SecretNetwork Message Encryption does not pad the ciphertext,
     * so the size of ciphertext is exactly same as the size of plaintext (UTF-8 encoded HandleMsg).
     *
     * Msg in the ContractExecute Transaction consists of the ciphertext and following 144 bytes data.
     * the data is 32(nonce) + 32(pubkey) + 64(contract code hash) + 16(AES-SIV S2V data)
     * about AES-SIV S2V data, see (https://datatracker.ietf.org/doc/html/rfc5297#section-2.4)
     * The size of Msg in the ContractExecute Transaction is 144 + X.
     */
    private padMsg(msg: object): object {
        const minimumPaddingSize = 7; // ,"p":""
        const msgSize = this.msgSize(msg);
        const surplus = msgSize % REQUEST_MSG_BLOCK_SIZE;
        const missing = REQUEST_MSG_BLOCK_SIZE - surplus;
        // if msgSize is multiple of 256 or missing less than minimumPaddingSize
        if (surplus === 0 || missing < minimumPaddingSize) {
            return msg;
        }
        const padded = {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            [Object.keys(msg)[0]]: {
                ...Object.values(msg)[0],
                p: ' '.repeat(missing - minimumPaddingSize),
            },
        };
        return padded;
    }

    /**
     * Returns byte length of msg in UTF-8 encoding.
     */
    private msgSize(msg: object): number {
        return JSON.stringify(msg).length;
    }

    public stdFee(gasLimit: number): StdFee {
        return {
            amount: [
                {
                    amount: (
                        Math.floor(gasLimit * this.gasPrice) + 1
                    ).toString(),
                    denom: 'uscrt',
                },
            ],
            gas: gasLimit.toString(),
        };
    }
}
