import fs from 'fs';
import { SecretNetworkClient, Wallet } from 'sbtc-js/node_modules/secretjs';
import {
    SigningCosmWasmClient,
    Secp256k1Pen,
    encodeSecp256k1Pubkey,
    pubkeyToAddress,
    EnigmaUtils,
} from 'secretjs';

export interface ContractReference {
    address: string;
    hash: string;
}

export interface DeployReport {
    timestamp: number;
    gitCommitHash: string;
    environment: string;
    deployerAddress: string;
    contracts: Record<string, ContractDeployReport>;
}

export interface ContractDeployReport {
    uploadTxHash: string;
    initTxHash: string;
    label: string;
    reference: ContractReference;
    initMsg: object | null;
}

const CONTRACTS_DIR_PATH = '../contracts';
const DEPLOY_REPORTS_PATH = '../deploy_reports';

export const waitForNode = async (client: SecretNetworkClient) => {
    let isNodeReady = false;
    while (!isNodeReady) {
        try {
            const account = await client.query.auth.account({
                address: client.address,
            });
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

export class ContractDeployer {
    public client: SecretNetworkClient;
    public environment: string;
    public timestamp: number;
    public gitCommitHash: string;
    public transactionWaitTime: number;
    public contractDeployReports: Record<string, ContractDeployReport> = {};
    public sc: SigningCosmWasmClient;

    constructor(
        client: SecretNetworkClient,
        environment: string,
        timestamp: number,
        gitCommitHash: string,
        transactionWaitTime: number,
        sc: SigningCosmWasmClient
    ) {
        this.client = client;
        this.environment = environment;
        this.timestamp = timestamp;
        this.gitCommitHash = gitCommitHash;
        this.transactionWaitTime = transactionWaitTime;
        this.sc = sc;
    }

    public static async init(
        mnemonic: string,
        grpcWebUrl: string,
        lcdURL: string,
        chainId: string,
        environment: string,
        gitCommitHash: string,
        transactionWaitTime: number
    ): Promise<ContractDeployer> {
        const wallet = new Wallet(mnemonic);
        const client = await SecretNetworkClient.create({
            grpcWebUrl,
            chainId,
            wallet,
            walletAddress: wallet.address,
        });
        const signingPen = await Secp256k1Pen.fromMnemonic(mnemonic);
        const pubkey = encodeSecp256k1Pubkey(signingPen.pubkey);
        const address = pubkeyToAddress(pubkey, 'secret');
        const sc = new SigningCosmWasmClient(
            lcdURL,
            address,
            (signBytes) => signingPen.sign(signBytes),
            EnigmaUtils.GenerateNewSeed(),
            {
                upload: {
                    amount: [{ amount: '62500', denom: 'uscrt' }],
                    gas: '5000000',
                },
                init: {
                    amount: [{ amount: '12500', denom: 'uscrt' }],
                    gas: '1000000',
                },
                exec: {
                    amount: [{ amount: '12500', denom: 'uscrt' }],
                    gas: '1000000',
                },
            }
        );
        console.log('waiting for node ...');
        await waitForNode(client);
        return new ContractDeployer(
            client,
            environment,
            Math.floor(Date.now() / 1000),
            gitCommitHash,
            transactionWaitTime,
            sc
        );
    }

    public async deployContract(
        contractDir: string,
        initMsg: object | null,
        contractName: string
    ): Promise<ContractReference> {
        console.group('Start Deployment: ', contractName);

        console.log('Uploading contract...');
        const wasm = this.loadWasm(contractDir);
        const uploadResult = await this.sc.upload(wasm);
        /*
        tx.compute.storeCode(
            {
                sender: this.client.address,
                wasmByteCode: wasm,
                source: '',
                builder: '',
            },
            { gasLimit: 10000000 }
        );
        */
        await this.wait();
        console.log('Uploaded', uploadResult);

        const codeId = uploadResult.codeId;
        console.log('Code id: ', codeId);
        const codeHash = await this.client.query.compute.codeHash(codeId);
        console.log('Contract hash: ', codeHash);

        console.log('Instantiating contract...');
        console.log(JSON.stringify(initMsg, null, 2));
        const label = this.buildContractLabel(contractName);
        const initResult = await this.client.tx.compute.instantiateContract(
            {
                sender: this.client.address,
                codeId,
                initMsg,
                label,
                codeHash: codeHash,
            },
            { gasLimit: 2000000 }
        );
        await this.wait();
        console.log('Instantiated', initResult);
        console.log('Contract deployed: ', { initResult, codeId, codeHash });
        console.groupEnd();

        const report = {
            uploadTxHash: uploadResult.transactionHash,
            initTxHash: initResult.transactionHash,
            label,
            reference: {
                address: initResult.jsonLog![0].events[1].attributes[0].value,
                hash: codeHash,
            },
            initMsg,
        };
        this.contractDeployReports[contractName] = report;
        return report.reference;
    }

    public async execute(contractInfo: ContractReference, msg: object) {
        console.group('Contract Execution');
        console.log(
            JSON.stringify(
                {
                    address: contractInfo.address,
                    msg: msg,
                },
                null,
                2
            )
        );
        console.log('Executing...');
        await this.client.tx.compute.executeContract(
            {
                sender: this.client.address,
                contractAddress: contractInfo.address,
                codeHash: contractInfo.hash,
                msg,
            },
            { gasLimit: 250000 }
        );
        console.log('Executed');
        console.groupEnd();
        await this.wait();
    }

    public exportDeployReport() {
        const filePath = `${DEPLOY_REPORTS_PATH}/${this.timestamp}.json`;
        const body = JSON.stringify(this.buildDeployReport(), null, 2);
        console.log('Deployed Report', body);
        console.log('Export Deployed Report To ', filePath);
        if (!fs.existsSync(DEPLOY_REPORTS_PATH)) {
            fs.mkdirSync(DEPLOY_REPORTS_PATH);
        }
        fs.writeFileSync(filePath, body);
    }

    private loadWasm(contractDir: string): Buffer {
        const wasmFilePath = `${CONTRACTS_DIR_PATH}/${contractDir}/contract.wasm`;
        console.log('Load wasm from: ', wasmFilePath);
        return fs.readFileSync(wasmFilePath);
    }

    private buildContractLabel(contractName: string): string {
        return `Shinobi_${this.environment}_${contractName}_${this.gitCommitHash}_${this.timestamp}`;
    }

    private buildDeployReport(): DeployReport {
        return {
            timestamp: this.timestamp,
            gitCommitHash: this.gitCommitHash,
            environment: this.environment,
            deployerAddress: this.client.address,
            contracts: this.contractDeployReports,
        };
    }

    private async wait(): Promise<void> {
        return new Promise((resolve) =>
            setTimeout(resolve, this.transactionWaitTime)
        );
    }
}
