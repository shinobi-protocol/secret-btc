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
            renderer.writeRow({
                executeResult: {
                    contractDetails: {
                        address: 'contract address1',
                        codeId: 0,
                        creator: 'creator address1',
                        label: 'label1',
                        initMsg: {},
                    },
                    msg: {
                        handle_msg: {},
                    },
                    transactionHash: 'transaction hash1',
                    answer: {},
                },
                txFeeResult: {
                    fee: {
                        amount: [],
                        gas: '200000',
                    },
                    gasUsed: 100000,
                },
            });
            renderer.writeRow({
                executeResult: {
                    contractDetails: {
                        address: 'contract address2',
                        codeId: 0,
                        creator: 'creator address2',
                        label: 'label2',
                        initMsg: {},
                    },
                    msg: {
                        handle_msg: {
                            body: 2000,
                        },
                    },
                    transactionHash: 'transaction hash',
                    answer: {},
                },
                txFeeResult: {
                    fee: {
                        amount: [],
                        gas: '200000',
                    },
                    gasUsed: 200000,
                },
                metadata: 'metadata string',
            });
            assert.equal(
                writer.string,
                [
                    '# Fee Report',
                    '',
                    '| Contract | Function | Message Length | Gas Used | Fee On SCRT (gasPrice = 0.25) | metadata |',
                    '| -------- | -------  | -------------: | -------: | ----------------------------: | -------- |',
                    '| label1 | handle_msg | 2 | 100,000 | 0.025 |  |',
                    '| label2 | handle_msg | 13 | 200,000 | 0.05 | metadata string |',
                ].join('\n')
            );
        });
    });
});
