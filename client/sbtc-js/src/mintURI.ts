import BigNumber from "bignumber.js";

// BIP-21
export function mintURI(mintAddress: string, amount: BigNumber): string {
    return 'bitcoin:' + mintAddress + '?amount=' + amount.toFixed();
}
