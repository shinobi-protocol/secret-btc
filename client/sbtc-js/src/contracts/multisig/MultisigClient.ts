/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/explicit-module-boundary-types */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-non-null-assertion */
import {
    HandleMsg,
    Convert,
    QueryMsg,
    QueryAnswer,
    MultisigStatus,
    TransactionStatus,
    PurpleCoin,
} from './types';
import { ContractClient, ExecuteResult } from '../ContractClient';

class MultisigClient extends ContractClient<HandleMsg, QueryMsg, QueryAnswer> {
    public async multisigStatus(): Promise<MultisigStatus> {
        return await this.query(
            {
                multisig_status: {},
            },
            (answer) => answer.multisig_status!
        );
    }

    public async transactionStatus(
        transactionId: number
    ): Promise<TransactionStatus> {
        return await this.query(
            {
                transaction_status: {
                    transaction_id: transactionId,
                },
            },
            (answer) => answer.transaction_status!
        );
    }

    // returns transaction Id
    public async submitTransaction(
        handleMsg: any,
        callbackCodeHash: string,
        contractAddress: string,
        sendFunds: PurpleCoin[]
    ): Promise<ExecuteResult<HandleMsg, number>> {
        const result = await this.execute(
            {
                submit_transaction: {
                    transaction: {
                        callback_code_hash: callbackCodeHash,
                        contract_addr: contractAddress,
                        msg: Buffer.from(
                            JSON.stringify(handleMsg),
                            'utf-8'
                        ).toString('base64'),
                        send: sendFunds,
                    },
                },
            },
            10000000,
            (answerJson: string) => {
                return Convert.toHandleAnswer(answerJson).submit_transaction
                    .transaction_id;
            }
        );
        return result;
    }

    public async signTransaction(transactionId: number): Promise<void> {
        await this.execute(
            {
                sign_transaction: { transaction_id: transactionId },
            },
            100000,
            () => void 0
        );
    }
}

export { MultisigClient };
