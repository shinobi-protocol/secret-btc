/* eslint-disable @typescript-eslint/ban-types */

import { ContractInfo, SecretNetworkClient, Tx } from 'secretjs';
import { Logger } from 'winston';

export class ExecuteError<HANDLE_MSG> extends Error {
    public tx: Tx;
    public contractAddress: string;
    public contractInfo: ContractInfo;
    public msg: HANDLE_MSG;
    public errorMsg: string;

    constructor(tx: Tx, contractAddress: string, contractInfo: ContractInfo, msg: HANDLE_MSG, errorMsg: string) {
        super(errorMsg);
        this.tx = tx;
        this.contractAddress = contractAddress;
        this.contractInfo = contractInfo;
        this.msg = msg;
        this.errorMsg = errorMsg;
    }
}

export interface ExecuteResult<HANDLE_MSG, ANSWER> {
    tx: Tx,
    contractAddress: string;
    contractInfo: ContractInfo;
    msg: HANDLE_MSG;
    answer: ANSWER;
}

interface PaddingMsg {
    [x: string]: any;
    p?: null | string;
}

const REQUEST_MSG_BLOCK_SIZE = 256;
const DEFAULT_GAS_PRICE = 0.25;

export abstract class ContractClient<
    HANDLE_MSG extends PaddingMsg,
    QUERY_MSG extends PaddingMsg,
    QUERY_ANSWER extends object
    > {
    public contractAddress: string;
    public contractInfo: Promise<ContractInfo>;
    public codeHash: Promise<string>;
    public secretNetworkClient: SecretNetworkClient;
    public gasPrice: number;
    protected logger: Logger;

    constructor(
        contractAddress: string,
        secretNetworkClient: SecretNetworkClient,
        logger: Logger,
        gasPrice = DEFAULT_GAS_PRICE
    ) {
        this.contractAddress = contractAddress;
        this.secretNetworkClient = secretNetworkClient;
        this.contractInfo = (async () => {
            let contractInfoResponse = await secretNetworkClient.query.compute.contractInfo(contractAddress);
            return contractInfoResponse.ContractInfo;
        })();
        this.codeHash =
            secretNetworkClient.query.compute.contractCodeHash(contractAddress)
        this.gasPrice = gasPrice;
        this.logger = logger;
    }

    public senderAddress(): string {
        return this.secretNetworkClient.address;
    }

    protected async execute<ANSWER>(
        msg: HANDLE_MSG,
        gasLimit: number,
        parseAnswer: (answerJson: string) => ANSWER
    ): Promise<ExecuteResult<HANDLE_MSG, ANSWER>> {
        const paddedMsg = this.padMsg(msg);
        this.logger.log({
            level: 'info',
            message:
                'Execute Start: ' +
                JSON.stringify(paddedMsg) +
                '\nGasLimit: ' + gasLimit
        });
        const result = await this.secretNetworkClient.tx.compute.executeContract(
            {
                sender: this.secretNetworkClient.address,
                contractAddress: this.contractAddress,
                codeHash: await this.codeHash,
                msg,
            }, {
            gasLimit
        }
        );
        if (result.code !== 0) {
            console.log(result);
            throw new ExecuteError(result, this.contractAddress, await this.contractInfo, msg, result.rawLog);
        }
        const answer = parseAnswer(new TextDecoder().decode(result.data[0] as Uint8Array));
        this.logger.log({
            level: 'info',
            message: `Execute Succeed\ntxhash: ${result.transactionHash}\ngasUsed/gasWanted: ${result.gasUsed}/${result.gasWanted}\nanswer:${JSON.stringify(answer)}`
        });
        return {
            tx: result,
            contractAddress: this.contractAddress,
            contractInfo: await this.contractInfo,
            msg,
            answer,
        };
    }

    protected async query(queryMsg: QUERY_MSG): Promise<QUERY_ANSWER> {
        const paddedMsg = this.padMsg(queryMsg);
        this.logger.log({
            level: 'info',
            message: 'Query Start: ' + JSON.stringify(paddedMsg),
        });
        const maxTry = 5;
        let error: Error;
        for (let tryCount = 0; tryCount < maxTry; tryCount++) {
            this.logger.log({
                level: 'info',
                message: `Query try ${tryCount + 1}/${maxTry}`,
            });
            // wait x^2 seconds;
            const waitTime = 1000 * (tryCount ^ 2);
            await new Promise((resolve) => setTimeout(resolve, waitTime));
            try {
                // Query
                const result =
                    await this.secretNetworkClient.query.compute.queryContract<QUERY_MSG, QUERY_ANSWER>({
                        contractAddress:
                            this.contractAddress,
                        codeHash: await this.codeHash,
                        query: paddedMsg,
                    }
                    );
                this.logger.log({
                    level: 'info',
                    message: 'Query Succeed: ' + JSON.stringify(result),
                });
                return result;
            } catch (e) {
                // catch error
                error = e as Error;
                this.logger.log({
                    level: 'info',
                    message: `Query catch error: ${error.message}`,
                });
            }
        }
        // After tried for maxTry times, throw the last error
        this.logger.log({
            level: 'info',
            message: 'Query throw error:',
        });
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        throw error!;
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
    private padMsg<T extends HANDLE_MSG | QUERY_MSG>(msg: T): T {
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
        return padded as T;
    }

    /**
     * Returns byte length of msg in UTF-8 encoding.
     */
    private msgSize<T extends HANDLE_MSG | QUERY_MSG>(msg: T): number {
        return JSON.stringify(msg).length;
    }


}
