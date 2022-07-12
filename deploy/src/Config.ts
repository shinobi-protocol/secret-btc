
export class Config {
    grpcWebUrl!: string;
    mnemonic!: string;
    rpcUrl!: string;
    environment!: string;
    gitCommitHash!: string;
    chainId!: string;
    transactionWaitTime!: number;

    public static init(): Config {
        let config = new Config();
        config.rpcUrl = process.env.TENDERMINT_RPC_URL!;
        config.grpcWebUrl = process.env.GRPC_WEB_URL!;
        config.mnemonic = process.env.MNEMONIC!;
        config.environment = process.env.ENVIRONMENT!;
        config.gitCommitHash = process.env.GIT_COMMIT_HASH!;
        config.chainId = process.env.CHAIN_ID!;
        config.transactionWaitTime = parseInt(process.env.TRANSACTION_WAIT_TIME!, 10);
        return config;
    }
}
