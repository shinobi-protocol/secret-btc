/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { config } from 'dotenv';
import { scheduleJob } from 'node-schedule';
import { addSeconds } from 'date-fns';
import BtcRpcClient from './BtcRpcClient';
import BtcClientInterface from './BtcClientInterface';
import RegtestUtilClient from './RegtestUtilClient';
import { TendermintRPCClient } from 'sbtc-js/build/TendermintRPCClient';
import { BitcoinSPVClient } from 'sbtc-js/build/contracts/bitcoin_spv/BitcoinSPVClient';
import { createLogger, transports, format, Logger } from 'winston';
import BtcSyncClient from './BtcSyncClient';
import TendermintSyncClient from './TendermintSyncClient';
import { SecretNetworkClient, Wallet } from 'sbtc-js/node_modules/secretjs';
import { SFPSClient } from 'sbtc-js/build/contracts/sfps/SFPSClient';
import { ShurikenClient } from 'sbtc-js/build/contracts/shuriken/ShurikenClient';

config({ path: process.env.ENV_FILE || '.env' });

const initSecretNetworkClient = async (): Promise<SecretNetworkClient> => {
    console.log('initializing secret network client...');
    const grpcWebUrl = process.env.GRPC_WEB_URL!;
    const mnemonic = process.env.MNEMONIC!;
    const chainId = process.env.CHAIN_ID!;
    console.log('grpcWebUrl', grpcWebUrl);
    const wallet = new Wallet(mnemonic);
    const client = await SecretNetworkClient.create({ grpcWebUrl, wallet, chainId, walletAddress: wallet.address });
    await client.query.auth.account({ address: client.address });
    console.log('Successfully connected to Secret Network');
    return client;
};

interface ContractClients {
    shurikenClient: ShurikenClient;
    bitcoinSPVClient: BitcoinSPVClient;
    sfpsClient: SFPSClient;
}

const initContractClients = async (
    secretNetworkClient: SecretNetworkClient,
    logger: Logger
): Promise<ContractClients> => {
    console.log('initializing contract clients...');
    const shurikenAddress = process.env.SHURIKEN_ADDRESS!;
    console.log('SHURIKEN_ADDRESS', shurikenAddress);

    const shurikenClient = new ShurikenClient(
        shurikenAddress,
        secretNetworkClient,
        logger
    );
    const config = await shurikenClient.config();
    console.log('Successfully connected to Shuriken Contract');
    const bitcoinSPVClient = new BitcoinSPVClient(
        config.bitcoin_spv.address,
        secretNetworkClient,
        logger
    );
    const sfpsClient = new SFPSClient(
        config.sfps.address,
        secretNetworkClient,
        logger
    );
    return { shurikenClient, bitcoinSPVClient, sfpsClient };
};

const initBtcClient = async (): Promise<BtcClientInterface> => {
    console.log('initializing btc client...');
    const apiType = process.env.BITCOIN_API_TYPE!;
    console.log('bitcoin api type:', apiType);
    let client: BtcClientInterface;
    switch (apiType) {
        case 'rpc': {
            const rpcUrl = process.env.BITCOIN_RPC_URL!;
            const rpcUser = process.env.BITCOIN_RPC_USER!;
            const rpcPass = process.env.BITCOIN_RPC_PASSWORD!;
            console.log('url:', rpcUrl);
            console.log('user:', rpcUser);
            console.log('pass:', rpcPass);
            client = new BtcRpcClient(rpcUrl, rpcUser, rpcPass);
            break;
        }
        case 'regtest_server': {
            const serverURL = process.env.BITCOIN_REGTEST_SERVER_URL;
            console.log('url:', serverURL);
            client = new RegtestUtilClient(
                serverURL || 'http://localhost:8080/1'
            );
            break;
        }
        default:
            throw new Error('invalid bitcoin api type');
    }
    await client.getBestBlockHeight();
    console.log('Successfully connected to Bitcoin');
    return client;
};

const initTendermintClient = (): TendermintRPCClient => {
    return new TendermintRPCClient(process.env.TENDERMINT_RPC_URL!);
};

const job = async (
    btcSyncClient: BtcSyncClient,
    tendermintSyncClient: TendermintSyncClient
) => {
    await btcSyncClient.syncBitcoinHeaders();
    await tendermintSyncClient.syncTendermintHeaders();
    scheduleJob(addSeconds(new Date(), 10), (): void => {
        void job(btcSyncClient, tendermintSyncClient);
    });
};

const printTitle = () => {
    console.log();
    console.log('       .__                 .__ __                  ');
    console.log('  _____|  |__  __ _________|__|  | __ ____   ____  ');
    console.log(' /  ___/  |  \\|  |  \\_  __ \\  |  |/ // __ \\ /    \\ ');
    console.log(' \\___ \\|   Y  \\  |  /|  | \\/  |    <\\  ___/|   |  \\');
    console.log('/____  >___|  /____/ |__|  |__|__|_ \\\\___  >___|  /');
    console.log('     \\/     \\/                     \\/    \\/     \\/ ');
    console.log();
};

const main = async () => {
    printTitle();
    const logger = createLogger({
        transports: [new transports.Console()],
        format: format.combine(format.simple(), format.timestamp()),
    });
    const btcClient = await initBtcClient();
    const secretNetworkClient = await initSecretNetworkClient();
    const {
        shurikenClient,
        bitcoinSPVClient,
        sfpsClient,
    } = await initContractClients(secretNetworkClient, logger);
    const tendermintClient = initTendermintClient();
    const btcSyncClient = new BtcSyncClient(
        shurikenClient,
        bitcoinSPVClient,
        btcClient,
        100,
        logger
    );
    const tendermintSyncClient = new TendermintSyncClient(
        shurikenClient,
        sfpsClient,
        tendermintClient,
        5,
        logger
    );
    await job(btcSyncClient, tendermintSyncClient);
};

main().catch((err) => {
    console.log('Exit process');
    console.log(err);
});
