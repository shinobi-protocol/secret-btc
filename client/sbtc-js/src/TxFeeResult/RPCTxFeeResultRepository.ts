import { RestClient } from "secretjs";
import FeeReport from "./TxFeeResult";
import TxFeeResultRepository from "./TxFeeResultRepository";

export default class RPCTxFeeResultRepository implements TxFeeResultRepository {
    private restClient: RestClient;

    constructor(restClient: RestClient) {
        this.restClient = restClient;
    }

    async get(txHash: string): Promise<FeeReport> {
        const tx = await this.restClient.txById(txHash);
        return {
            fee: tx.tx.value.fee,
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            gasUsed: parseInt(tx.gas_used!),
        }
    }
}