import {
    FeeTable,
    Secp256k1Pen,
    encodeSecp256k1Pubkey,
    pubkeyToAddress,
    EnigmaUtils,
    SigningCosmWasmClient,
} from 'secretjs';

export const buildClient = async (
    mnemonic: string,
    lcdUrl: string,
    customFees?: Partial<FeeTable>
) => {
    // A pen is the most basic tool you can think of for signing.
    // This wraps a single keypair and allows for signing.
    const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic);

    // Get the public key
    const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);

    // get the wallet address
    const deployerAddress = pubkeyToAddress(pubkey, 'secret');

    const txEncryptionSeed = EnigmaUtils.GenerateNewSeed();

    return {
        client: new SigningCosmWasmClient(
            lcdUrl,
            deployerAddress,
            (signBytes) => signingPen.sign(signBytes),
            txEncryptionSeed,
            customFees
        ),
        deployerAddress: deployerAddress,
    };
};

export const waitForNode = async (
    client: SigningCosmWasmClient,
    deployerAddress: string
) => {
    let isNodeReady = false;
    while (!isNodeReady) {
        try {
            const account = await client.getAccount(deployerAddress);
            if (account !== undefined) {
                console.log('node is ready');
                isNodeReady = true;
            }
        } catch (_) {
        } finally {
            await new Promise((resolve) => setTimeout(resolve, 1000));
        }
    }
};
