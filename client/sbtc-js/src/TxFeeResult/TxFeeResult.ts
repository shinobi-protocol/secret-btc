import { StdFee } from "secretjs/types/types";

export default interface TxFeeResult {
    fee: StdFee,
    gasUsed: number,
}