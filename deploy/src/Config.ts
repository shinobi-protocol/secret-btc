import { FeeTable } from 'secretjs';

export class Config {
    lcdUrl!: string;
    mnemonic!: string;
    rpcUrl!: string;
    environment!: string;
    gitCommitHash!: string;
    customFees?: Partial<FeeTable>;

    public static init(): Config {
        let config = new Config();
        config.rpcUrl = process.env.TENDERMINT_RPC_URL!;
        config.lcdUrl = process.env.LCD_URL!;
        config.mnemonic = process.env.MNEMONIC!;
        config.environment = process.env.ENVIRONMENT!;
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
