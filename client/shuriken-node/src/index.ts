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
import { BitcoinSPVClient } from 'sbtc-js/build/contracts/bitcoin_spv/BitcoinSPVClient';
import { createLogger, transports, format, Logger } from 'winston';
import BtcSyncClient from './BtcSyncClient';
import SFPSSyncClient from './SFPSSyncClient';
import { Wallet } from 'sbtc-js/node_modules/secretjs';
import { SFPSClient } from 'sbtc-js/build/contracts/sfps/SFPSClient';
import { ShurikenClient } from 'sbtc-js/build/contracts/shuriken/ShurikenClient';
import ShinobiClient from 'sbtc-js/build/ShinobiClient';

config({ path: process.env.ENV_FILE || '.env' });

const initShinobiClient = async (): Promise<ShinobiClient> => {
    console.log('initializing shinobi client...');
    const grpcWebUrl = process.env.GRPC_WEB_URL!;
    const mnemonic = process.env.MNEMONIC!;
    const chainId = process.env.CHAIN_ID!;
    console.log('grpcWebUrl', grpcWebUrl);
    const wallet = new Wallet(mnemonic);
    const client = await ShinobiClient.create(
        grpcWebUrl,
        chainId,
        wallet,
        wallet.address
    );
    await client.sn.query.auth.account({ address: client.sn.address });
    console.log('Successfully connected to Secret Network');
    return client;
};

interface ContractClients {
    shurikenClient: ShurikenClient;
    bitcoinSPVClient: BitcoinSPVClient;
    sfpsClient: SFPSClient;
}

const initContractClients = async (
    shinoboClient: ShinobiClient,
    logger: Logger
): Promise<ContractClients> => {
    console.log('initializing contract clients...');
    const shurikenAddress = process.env.SHURIKEN_ADDRESS!;
    console.log('SHURIKEN_ADDRESS', shurikenAddress);

    const shurikenClient = new ShurikenClient(
        shurikenAddress,
        shinoboClient,
        logger
    );
    const config = await shurikenClient.config();
    console.log('Successfully connected to Shuriken Contract');
    const bitcoinSPVClient = new BitcoinSPVClient(
        config.bitcoin_spv.address,
        shinoboClient,
        logger
    );
    const sfpsClient = new SFPSClient(
        config.sfps.address,
        shinoboClient,
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

const job = async (
    btcSyncClient: BtcSyncClient,
    sfpsSyncClient: SFPSSyncClient
) => {
    await btcSyncClient.syncBitcoinHeaders();
    await sfpsSyncClient.syncSFPSHeaders();
    scheduleJob(addSeconds(new Date(), 10), (): void => {
        void job(btcSyncClient, sfpsSyncClient);
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
    const shinobiClient = await initShinobiClient();
    const {
        shurikenClient,
        bitcoinSPVClient,
        sfpsClient,
    } = await initContractClients(shinobiClient, logger);
    const btcSyncClient = new BtcSyncClient(
        shurikenClient,
        bitcoinSPVClient,
        btcClient,
        100,
        logger
    );
    const sfpsSyncClient = new SFPSSyncClient(
        shurikenClient,
        sfpsClient,
        parseInt(process.env.SFPS_BLOCK_PER_TX!),
        logger
    );
    await job(btcSyncClient, sfpsSyncClient);
};

main().catch((err) => {
    console.log('Exit process');
    console.log(err);
});
