import { Config } from './Config';
import { ContractReference, ContractDeployer } from './ContractDeployer';
import * as crypto from 'crypto';
import BigNumber from 'bignumber.js';
import { Header } from 'sbtc-js/node_modules/secretjs/dist/protobuf_stuff/tendermint/types/types';

export const deployMultisig = async (
    deployer: ContractDeployer
): Promise<ContractReference> => {
    return deployer.deployContract(
        'multisig',
        {
            config: {
                signers: [deployer.client.address],
                required: 1,
            },
        },
        'MULTISIG'
    );
};

export const deployLog = async (
    deployer: ContractDeployer
): Promise<ContractReference> => {
    return await deployer.deployContract(
        'log',
        {
            entropy: crypto.randomBytes(32).toString('base64'),
        },
        'LOG'
    );
};

export const deployBitcoinSPV = async (
    deployer: ContractDeployer,
    state_proxy: ContractReference
): Promise<ContractReference> => {
    const initMsg = {
        ...require('./init_msg/bitcoin_spv.json')[deployer.environment],
        state_proxy,
        seed: crypto.randomBytes(32).toString('base64'),
    };
    return deployer.deployContract('bitcoin_spv', initMsg, 'BITCOIN_SPV');
};

export const deploySFPS = async (
    deployer: ContractDeployer,
    state: ContractReference,
): Promise<ContractReference> => {
    const initialSnHeader = (
        await deployer.client.query.tendermint.getLatestBlock({})
    ).block!.header!;
    console.log(JSON.stringify(initialSnHeader));
    const entropy = Buffer.from(crypto.randomBytes(32));
    const initMsg = {
        ...require('./init_msg/sfps.json')[deployer.environment],
        initial_header: Buffer.from(
            Header.encode(initialSnHeader).finish()
        ).toString('base64'),
        entropy: entropy.toString('base64'),
        seed: entropy.toString('base64'),
        config: {
            state_proxy: state,
        }
    };
    return deployer.deployContract('sfps', initMsg, 'SFPS');
};

export const deploySBTC = async (
    deployer: ContractDeployer
): Promise<ContractReference> => {
    const initMsg = {
        prng_seed: crypto.randomBytes(32).toString('base64'),
        ...require('./init_msg/sbtc.json')[deployer.environment],
    };
    return deployer.deployContract('token', initMsg, 'SBTC');
};

export const deploySNB = async (
    deployer: ContractDeployer
): Promise<ContractReference> => {
    const initMsg = {
        initial_balances: [
            {
                address: deployer.client.address,
                amount: new BigNumber(1_000_000).shiftedBy(8),
            },
        ],
        prng_seed: crypto.randomBytes(32).toString('base64'),
        ...require('./init_msg/snb.json')[deployer.environment],
    };
    return deployer.deployContract('token', initMsg, 'SNB');
};

export const deployGateway = async (
    deployer: ContractDeployer,
    bitcoinSPV: ContractReference,
    sfps: ContractReference,
    sbtc: ContractReference,
    log: ContractReference,
    multisig: ContractReference,
    state: ContractReference
): Promise<ContractReference> => {
    const initMsg = {
        seed: crypto.randomBytes(32).toString('base64'),
        config: {
            bitcoin_spv: bitcoinSPV,
            sfps: sfps,
            sbtc: sbtc,
            log: log,
            state_proxy: state,
            owner: multisig.address,
            ...require('./init_msg/gateway.json')[deployer.environment].config,
        },
    };
    const gateway = await deployer.deployContract(
        'gateway',
        initMsg,
        'GATEWAY'
    );
    await setOwnerOfToken(deployer, sbtc, gateway.address);
    return gateway;
};

export const deployShuriken = async (
    deployer: ContractDeployer,
    bitcoinSPV: ContractReference,
    sfps: ContractReference,
    state_proxy: ContractReference
): Promise<ContractReference> => {
    const initMsg = {
        config: {
            bitcoin_spv: bitcoinSPV,
            sfps: sfps,
            state_proxy: state_proxy
        },
        seed: crypto.randomBytes(32).toString('base64'),
    };
    return deployer.deployContract('shuriken', initMsg, 'SHURIKEN');
};

export const deployState = async (
    deployer: ContractDeployer
): Promise<ContractReference> => {
    return await deployer.deployContract(
        'state',
        { contract_owners: [] },
        'STATE'
    );
};

export const setupLog = async (
    deployer: ContractDeployer,
    log: ContractReference,
    gateway: ContractReference
): Promise<void> => {
    return await deployer.execute(log, {
        setup: {
            config: {
                gateway: gateway,
            },
        },
    });
};

export const setOwnerOfToken = async (
    deployer: ContractDeployer,
    token: ContractReference,
    owner: String
) => {
    await deployer.execute(token, {
        set_minters: {
            minters: [owner],
        },
    });
    await deployer.execute(token, {
        change_admin: {
            address: owner,
        },
    });
};

export const deployVesting = async (deployer: ContractDeployer) => {
    await deployer.deployContract('vesting', {}, 'VESTING');
};

export const main = async () => {
    const config = Config.init();
    console.log('config', config);
    const deployer = await ContractDeployer.init(
        config.mnemonic,
        config.grpcWebUrl,
        config.lcdUrl,
        config.chainId,
        config.environment,
        config.gitCommitHash,
        config.transactionWaitTime
    );
    const state = await deployState(deployer);
    const multisig = await deployMultisig(deployer);
    const log = await deployLog(deployer);
    const bitcoinSPV = await deployBitcoinSPV(deployer, state);
    const sfps = await deploySFPS(deployer, state);
    const sbtc = await deploySBTC(deployer);
    await deploySNB(deployer);
    const gateway = await deployGateway(
        deployer,
        bitcoinSPV,
        sfps,
        sbtc,
        log,
        multisig,
        state
    );
    await deployShuriken(deployer, bitcoinSPV, sfps, state);
    await deployVesting(deployer);
    await setupLog(deployer, log, gateway);
    deployer.exportDeployReport();
    console.log('--DEPLOYMENT SUCCESS--');
};
