import { FeeTable } from 'secretjs';

export class Config {
    bitcoinNetwork!: string;
    lcdUrl!: string;
    mnemonic!: string;
    rpcUrl!: string;
    snNetwork!: string;
    gitCommitHash!: string;
    customFees?: Partial<FeeTable>;

    public static init(): Config {
        let config = new Config();
        config.bitcoinNetwork = process.env.BITCOIN_NETWORK!;
        config.rpcUrl = process.env.TENDERMINT_RPC_URL!;
        config.lcdUrl = process.env.LCD_URL!;
        config.mnemonic = process.env.MNEMONIC!;
        config.snNetwork = process.env.SN_NETWORK!;
        config.gitCommitHash = process.env.GIT_COMMIT_HASH!;
        config.customFees = {
            upload: {
                amount: [{ amount: '1250000', denom: 'uscrt' }],
                gas: '5000000',
            },
            init: {
                amount: [{ amount: '250000', denom: 'uscrt' }],
                gas: '1000000',
            },
            exec: {
                amount: [{ amount: '250000', denom: 'uscrt' }],
                gas: '1000000',
            },
        };
        return config;
    }
}
