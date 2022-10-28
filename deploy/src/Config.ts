export class Config {
    grpcWebUrl!: string;
    mnemonic!: string;
    environment!: string;
    gitCommitHash!: string;
    chainId!: string;
    transactionWaitTime!: number;
    lcdUrl!: string;

    public static init(): Config {
        let config = new Config();
        config.grpcWebUrl = process.env.GRPC_WEB_URL!;
        config.lcdUrl = process.env.LCD_URL!;
        config.mnemonic = process.env.MNEMONIC!;
        config.environment = process.env.ENVIRONMENT!;
        config.gitCommitHash = process.env.GIT_COMMIT_HASH!;
        config.chainId = process.env.CHAIN_ID!;
        config.transactionWaitTime = parseInt(
            process.env.TRANSACTION_WAIT_TIME!
        );
        return config;
    }
}
