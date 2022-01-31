import { buildSigningCosmWasmClient, winston } from 'sbtc-js';
import { MultisigClient } from 'sbtc-js/build/contracts/multisig/MultisigClient';
import { Config } from './config';

export async function buildClient(config: Config): Promise<MultisigClient> {
    const signingCosmwasmClient = await buildSigningCosmWasmClient(
        config.lcdURL,
        config.mnemonic
    );
    return new MultisigClient(
        config.multisigAddress,
        signingCosmwasmClient,
        winston.createLogger({
            level: process.env.DEBUG ? 'info' : 'error',
            format: winston.format.simple(),
            transports: [new winston.transports.Console()],
        })
    );
}
