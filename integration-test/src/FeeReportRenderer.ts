import * as fs from 'fs';
import { ExecuteResult } from 'sbtc-js/build/contracts/ContractClient';


export interface Writer {
    write(str: string): void;
}

export class FileWriter implements Writer {
    private fd: number;
    constructor(path: string) {
        this.fd = fs.openSync(path, 'w');
    }
    public write(str: string): void {
        fs.writeSync(this.fd, str);
    }
}

export class FeeReportRenderer {
    private writer: Writer;

    constructor(writer: Writer) {
        this.writer = writer;
        this.writeHeader();
    }

    private writeHeader(): void {
        this.writer.write(
            [
                '# Fee Report',
                '',
                '| Contract | Function | Message Length | Gas Used | Fee On SCRT (gasPrice = 0.25) |',
                '| -------- | -------  | -------------: | -------: | ----------------------------: |',
            ].join('\n')
        );
    }

    public writeRow(executeResult: ExecuteResult<any, any>): void {
        this.writer.write(this.executeResultToRow(executeResult));
    }

    private executeResultToRow(executeResult: ExecuteResult<any, any>): string {
        const contract = executeResult.contractInfo.label;
        const functionName = Object.keys(executeResult.msg)[0];
        const messageLength = JSON.stringify(
            Object.values(executeResult.msg)[0]
        ).length;
        const gasUsed = executeResult.tx.gasUsed.toLocaleString();
        const feeOnSCRT = (
            (executeResult.tx.gasUsed * 0.25) /
            10 ** 6
        ).toLocaleString();
        return `\n| ${contract} | ${functionName} | ${messageLength} | ${gasUsed} | ${feeOnSCRT} |`;
    }
}
