import { SigningCosmWasmClient, FeeTable } from 'secretjs';
import fs from 'fs';
import { buildClient, waitForNode } from './buildClient';

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
    initMsg: object;
}

const CONTRACTS_DIR_PATH = '../contracts';
const DEPLOY_REPORTS_PATH = '../deploy_reports';

export class ContractDeployer {
    public client: SigningCosmWasmClient;
    public environment: string;
    public timestamp: number;
    public gitCommitHash: string;
    public contractDeployReports: Record<string, ContractDeployReport> = {};

    constructor(
        client: SigningCosmWasmClient,
        environment: string,
        timestamp: number,
        gitCommitHash: string
    ) {
        this.client = client;
        this.environment = environment;
        this.timestamp = timestamp;
        this.gitCommitHash = gitCommitHash;
    }

    public static async init(
        mnemonic: string,
        lcdUrl: string,
        environment: string,
        gitCommitHash: string,
        customFees?: Partial<FeeTable>
    ): Promise<ContractDeployer> {
        const { client, deployerAddress } = await buildClient(
            mnemonic,
            lcdUrl,
            customFees
        );
        console.log('waiting for node ...');
        await waitForNode(client, deployerAddress);
        return new ContractDeployer(
            client,
            environment,
            Math.floor(Date.now() / 1000),
            gitCommitHash
        );
    }

    public async deployContract(
        contractDir: string,
        initMsg: object,
        contractName: string
    ): Promise<ContractReference> {
        console.group('Start Deployment: ', contractName);

        console.log('Uploading contract...');
        const wasm = this.loadWasm(contractDir);
        const uploadReceipt = await this.client.upload(wasm, {});
        await this.wait();
        console.log('Uploaded');

        const codeId = uploadReceipt.codeId;
        console.log('Code id: ', codeId);
        const hash = await this.client.getCodeHashByCodeId(codeId);
        console.log('Contract hash: ', hash);

        console.log('Instantiating contract...');
        console.log(JSON.stringify(initMsg, null, 2));
        const label = this.buildContractLabel(contractName);
        const initReceipt = await this.client.instantiate(
            codeId,
            initMsg,
            label
        );
        await this.wait();
        console.log('Instantiated');
        console.log('Contract deployed: ', { initReceipt, codeId, hash });
        console.groupEnd();

        const report = {
            uploadTxHash: uploadReceipt.transactionHash,
            initTxHash: initReceipt.transactionHash,
            label,
            reference: { address: initReceipt.contractAddress, hash },
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
        await this.client.execute(contractInfo.address, msg);
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
            deployerAddress: this.client.senderAddress,
            contracts: this.contractDeployReports,
        };
    }

    private async wait(): Promise<void> {
        return new Promise((resolve) => setTimeout(resolve, 5000));
    }
}
