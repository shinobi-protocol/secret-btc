import TxFeeResult from 'sbtc-js/build/TxFeeResult/TxFeeResult';
import { ExecuteResult } from 'sbtc-js/build/contracts/ContractClient';
import * as fs from 'fs';

export interface FeeReport {
    executeResult: ExecuteResult<any, any>;
    txFeeResult: TxFeeResult;
    metadata?: string;
}

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
                '| Contract | Function | Message Length | Gas Used | Fee On SCRT (gasPrice = 0.25) | metadata |',
                '| -------- | -------  | -------------: | -------: | ----------------------------: | -------- |',
            ].join('\n')
        );
    }

    public writeRow(feeReport: FeeReport): void {
        this.writer.write(this.feeReportToRow(feeReport));
    }

    private feeReportToRow(feeReport: FeeReport): string {
        const contract = feeReport.executeResult.contractDetails.label;
        const functionName = Object.keys(feeReport.executeResult.msg)[0];
        const messageLength = JSON.stringify(
            Object.values(feeReport.executeResult.msg)[0]
        ).length;
        const gasUsed = feeReport.txFeeResult.gasUsed.toLocaleString();
        const feeOnSCRT = (
            (feeReport.txFeeResult.gasUsed * 0.25) /
            10 ** 6
        ).toLocaleString();
        const metadata = feeReport.metadata || '';
        return `\n| ${contract} | ${functionName} | ${messageLength} | ${gasUsed} | ${feeOnSCRT} | ${metadata} |`;
    }
}
