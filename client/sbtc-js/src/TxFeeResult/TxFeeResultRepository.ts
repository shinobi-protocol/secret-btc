import TxFeeResultj from "./TxFeeResult";

export default interface TxFeeResult {
    get(txHash: string): Promise<TxFeeResultj>;
}