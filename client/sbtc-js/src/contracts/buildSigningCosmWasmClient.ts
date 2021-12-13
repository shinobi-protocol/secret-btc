import {
    FeeTable,
    SigningCosmWasmClient,
    Secp256k1Pen,
    encodeSecp256k1Pubkey,
    pubkeyToAddress,
    EnigmaUtils,
} from 'secretjs';

export async function buildSigningCosmWasmClient(
    apiUrl: string,
    mnemonic: string,
    // TODO edit custom fees
    customFees: Partial<FeeTable> = {
        exec: {
            amount: [{ amount: '500000', denom: 'uscrt' }],
            gas: '500000',
        },
    },
    txEncryptionSeed?: Uint8Array
): Promise<SigningCosmWasmClient> {
    const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic);
    const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);
    const address = pubkeyToAddress(pubkey, 'secret');
    return new SigningCosmWasmClient(
        apiUrl,
        address,
        (signBytes) => signingPen.sign(signBytes),
        txEncryptionSeed || EnigmaUtils.GenerateNewSeed(),
        customFees
    );
}
