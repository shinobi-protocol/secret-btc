import { assert } from 'chai';
import { describe } from 'mocha';
import { FeeReportRenderer, Writer } from '../src/FeeReportRenderer';

class MockWriter implements Writer {
    public string = '';
    write(str: string): void {
        this.string += str;
    }
}

describe('FeeReportRenderer', () => {
    describe('render fee reports', () => {
        it('renders markdown text', () => {
            const writer = new MockWriter();
            const renderer = new FeeReportRenderer(writer);
            renderer.writeRow(
                {
                    tx: {
                        height: 0,
                        timestamp: 'timestamp1',
                        transactionHash: 'transaction hash1',
                        code: 0,
                        rawLog: 'raw log1',
                        events: [],
                        data: [],
                        tx: {
                            body: {

                                messages: [],
                                memo: '',
                                timeoutHeight: '',
                                extensionOptions: [],
                                nonCriticalExtensionOptions: []
                            },
                            authInfo: {
                                signerInfos: [],
                            },
                            signatures: []
                        },
                        txBytes: Buffer.from([]),
                        gasUsed: 100000,
                        gasWanted: 0,
                    },
                    contractAddress: 'contract address1',
                    contractInfo: {
                        codeId: '',
                        creator: 'creator address1',
                        label: 'label1',
                    },
                    msg: {
                        handle_msg: {},
                    },
                    answer: {},
                },
            );
            renderer.writeRow(
                {
                    tx: {
                        height: 0,
                        timestamp: 'timestamp2',
                        transactionHash: 'transaction hash2',
                        code: 0,
                        rawLog: 'raw log1',
                        events: [],
                        data: [],
                        tx: {
                            body: {

                                messages: [],
                                memo: '',
                                timeoutHeight: '',
                                extensionOptions: [],
                                nonCriticalExtensionOptions: []
                            },
                            authInfo: {
                                signerInfos: [],
                            },
                            signatures: []
                        },
                        txBytes: Buffer.from([]),
                        gasUsed: 200000,
                        gasWanted: 0,
                    },
                    contractAddress: 'contract address2',
                    contractInfo: {
                        codeId: '',
                        creator: 'creator address2',
                        label: 'label2',
                    },
                    msg: {
                        handle_msg: {
                            body: 2000
                        },
                    },
                    answer: {},
                },
            );
            assert.equal(
                writer.string,
                [
                    '# Fee Report',
                    '',
                    '| Contract | Function | Message Length | Gas Used | Fee On SCRT (gasPrice = 0.25) |',
                    '| -------- | -------  | -------------: | -------: | ----------------------------: |',
                    '| label1 | handle_msg | 2 | 100,000 | 0.025 |',
                    '| label2 | handle_msg | 13 | 200,000 | 0.05 |',
                ].join('\n')
            );
        });
    });
});
