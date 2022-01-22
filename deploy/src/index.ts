import { Config } from './Config';
import axios from 'axios';
import { ContractReference, ContractDeployer } from './ContractDeployer';
import * as crypto from 'crypto';
import BigNumber from 'bignumber.js';

const deployMultisig = async (
    deployer: ContractDeployer
): Promise<ContractReference> => {
    return deployer.deployContract(
        'multisig',
        {
            config: {
                signers: [deployer.client.senderAddress],
                required: 1,
            },
        },
        'MULTISIG'
    );
};

const deployLog = async (
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

const deployBitcoinSPV = async (
    deployer: ContractDeployer
): Promise<ContractReference> => {
    const initMsg = {
        ...require('./init_msg/bitcoin_spv.json')[deployer.environment],
    };
    return deployer.deployContract('bitcoin_spv', initMsg, 'BITCOIN_SPV');
};

const deploySFPS = async (
    deployer: ContractDeployer,
    rpcUrl: string
): Promise<ContractReference> => {
    const initialSnHeader = (await axios(rpcUrl + '/block')).data.result.block
        .header;
    console.log(JSON.stringify(initialSnHeader));
    const entropy = Buffer.from(crypto.randomBytes(32));
    const initMsg = {
        ...require('./init_msg/sfps.json')[deployer.environment],
        initial_header: initialSnHeader,
        entropy: entropy.toString('base64'),
    };
    return deployer.deployContract('sfps', initMsg, 'SFPS');
};

const deploySBTC = async (
    deployer: ContractDeployer
): Promise<ContractReference> => {
    const initMsg = {
        prng_seed: crypto.randomBytes(32).toString('base64'),
        ...require('./init_msg/sbtc.json')[deployer.environment],
    };
    return deployer.deployContract('token', initMsg, 'SBTC');
};

const deploySNB = async (
    deployer: ContractDeployer
): Promise<ContractReference> => {
    const initMsg = {
        initial_balances: [
            {
                address: deployer.client.senderAddress,
                amount: new BigNumber(1_000_000_000).shiftedBy(8),
            },
        ],
        prng_seed: crypto.randomBytes(32).toString('base64'),
        ...require('./init_msg/snb.json')[deployer.environment],
    };
    return deployer.deployContract('token', initMsg, 'SNB');
};

const deployGateway = async (
    deployer: ContractDeployer,
    bitcoinSPV: ContractReference,
    sfps: ContractReference,
    sbtc: ContractReference,
    log: ContractReference,
    multisig: ContractReference
): Promise<ContractReference> => {
    const initMsg = {
        entropy: crypto.randomBytes(32).toString('base64'),
        config: {
            bitcoin_spv: bitcoinSPV,
            sfps: sfps,
            sbtc: sbtc,
            finance_admin: {
                address: deployer.client.senderAddress,
                hash: '',
            },
            log: log,
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

const deployTreasury = async (
    deployer: ContractDeployer,
    snb: ContractReference,
    log: ContractReference
) => {
    const initMsg = {
        config: {
            owner: deployer.client.senderAddress,
            snb: snb,
            log: log,
        },
    };
    const treasury = await deployer.deployContract(
        'treasury',
        initMsg,
        'TREASURY'
    );
    await deployer.execute(snb, {
        transfer: {
            recipient: treasury.address,
            amount: new BigNumber(950_000_000).shiftedBy(8),
        },
    });
    return treasury;
};

const deployShuriken = async (
    deployer: ContractDeployer,
    bitcoinSPV: ContractReference,
    sfps: ContractReference
): Promise<ContractReference> => {
    const initMsg = {
        config: {
            finance_admin: {
                address: deployer.client.senderAddress,
                hash: '',
            },
            bitcoin_spv: bitcoinSPV,
            sfps: sfps,
        },
    };
    return deployer.deployContract('shuriken', initMsg, 'SHURIKEN');
};

const deployFinanceAdminV1 = async (
    deployer: ContractDeployer,
    gateway: ContractReference,
    treasury: ContractReference,
    shuriken: ContractReference,
    snb: ContractReference
): Promise<ContractReference> => {
    const initMsg = {
        config: {
            owner: deployer.client.senderAddress,
            gateway: gateway,
            treasury: treasury,
            shuriken: shuriken,
            snb: snb,
            developer_address: deployer.client.senderAddress,
        },
        ...require('./init_msg/finance_admin.json'),
    };
    const financeAdmin = await deployer.deployContract(
        'finance_admin_v1',
        initMsg,
        'FINANCE_ADMIN_V1'
    );
    await deployer.execute(gateway, {
        change_finance_admin: {
            new_finance_admin: financeAdmin,
        },
    });
    await deployer.execute(shuriken, {
        change_finance_admin: {
            new_finance_admin: financeAdmin,
        },
    });
    await deployer.execute(treasury, {
        transfer_ownership: {
            owner: financeAdmin.address,
        },
    });
    await setOwnerOfToken(deployer, snb, financeAdmin.address);
    return financeAdmin;
};

const setupLog = async (
    deployer: ContractDeployer,
    log: ContractReference,
    gateway: ContractReference,
    treasury: ContractReference
): Promise<void> => {
    return await deployer.execute(log, {
        setup: {
            config: {
                gateway: gateway,
                treasury: treasury,
            },
        },
    });
};

const setOwnerOfToken = async (
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

const main = async () => {
    const config = Config.init();
    console.log('config', config);
    const deployer = await ContractDeployer.init(
        config.mnemonic,
        config.lcdUrl,
        config.environment,
        config.gitCommitHash,
        config.customFees
    );
    const multisig = await deployMultisig(deployer);
    const log = await deployLog(deployer);
    const bitcoinSPV = await deployBitcoinSPV(deployer);
    const sfps = await deploySFPS(deployer, config.rpcUrl);
    const sbtc = await deploySBTC(deployer);
    const snb = await deploySNB(deployer);
    const gateway = await deployGateway(
        deployer,
        bitcoinSPV,
        sfps,
        sbtc,
        log,
        multisig
    );
    const treasury = await deployTreasury(deployer, snb, log);
    const shuriken = await deployShuriken(deployer, bitcoinSPV, sfps);
    await deployFinanceAdminV1(deployer, gateway, treasury, shuriken, snb);
    await setupLog(deployer, log, gateway, treasury);
    deployer.exportDeployReport();
    console.log('--DEPLOYMENT SUCCESS--');
};

main().catch(console.error);
