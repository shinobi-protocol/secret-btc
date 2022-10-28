import {
    merkleRoot,
    innerHash,
    leafHash,
    getSplitPoint,
    MerkleProof,
} from '../src/contracts/sfps/TendermintMerkleTree';
import { assert } from 'chai';
import { describe, it } from 'mocha';
import { encodeToBuffer } from '../src/proto';
import { ResponseDeliverTx } from 'secretjs/dist/protobuf_stuff/tendermint/abci/types';

const responseDeliverTx: ResponseDeliverTx = {
    code: 0,
    data: Buffer.from(
        'CpcDCiovc2VjcmV0LmNvbXB1dGUudjFiZXRhMS5Nc2dFeGVjdXRlQ29udHJhY3QS6AJVzmyAo99HQULa7zPh5czf0jm6vUIgLaV5GqHcbtdXEEIK0ZGQvykkyg6ikiUk3o5ynhv8kWK/3I+Eg9FEESWZuBsML2e4q/a4tFJk6eogjMhfJQ2uO4SKdPg4NTjB0An4bCC1uQdryNwQOpRM//GgidK55QZCeVIfEOAaD1GdVomol5t12V2qbjuM2vk/U8OLuLdjGgV2aMxju5tmjkdfwLiw2EpJlOG4qkuF9Lef0IUiFI627ED7G7JM2bzjjeI3ihXbYEgmmBfIe6gExN6K7yrJnWjzxrSVp3wAAkDkHu968GE+nX/mAuMyW7ME3yAnRv29wbyGgQi6tOByLIcKPy/1c3j3PEkmV2PQ/003zh5giCRJ75vbBs6xKcOM2V+l6m22N0KjVZZGfpJyOdJLQQPr7DYAQYarMEO81NayHr/SBjDbfOcfZoFcosg+dr7xfzSqqegs81dZUBSwsvboa0jILtzzbOE=',
        'base64'
    ),
    log: '',
    info: '',
    gasUsed: '23757',
    gasWanted: '50000',
    events: [],
    codespace: '',
};

describe('TendermintMerkleTree', () => {
    describe('ResponseDeliverTx', () => {
        describe('encode', () => {
            it('should encode response deliver tx correctly', () => {
                const responseDeliverTx = {
                    code: 0,
                    data: Buffer.from(
                        'CpcDCiovc2VjcmV0LmNvbXB1dGUudjFiZXRhMS5Nc2dFeGVjdXRlQ29udHJhY3QS6AJVzmyAo99HQULa7zPh5czf0jm6vUIgLaV5GqHcbtdXEEIK0ZGQvykkyg6ikiUk3o5ynhv8kWK/3I+Eg9FEESWZuBsML2e4q/a4tFJk6eogjMhfJQ2uO4SKdPg4NTjB0An4bCC1uQdryNwQOpRM//GgidK55QZCeVIfEOAaD1GdVomol5t12V2qbjuM2vk/U8OLuLdjGgV2aMxju5tmjkdfwLiw2EpJlOG4qkuF9Lef0IUiFI627ED7G7JM2bzjjeI3ihXbYEgmmBfIe6gExN6K7yrJnWjzxrSVp3wAAkDkHu968GE+nX/mAuMyW7ME3yAnRv29wbyGgQi6tOByLIcKPy/1c3j3PEkmV2PQ/003zh5giCRJ75vbBs6xKcOM2V+l6m22N0KjVZZGfpJyOdJLQQPr7DYAQYarMEO81NayHr/SBjDbfOcfZoFcosg+dr7xfzSqqegs81dZUBSwsvboa0jILtzzbOE=',
                        'base64'
                    ),
                    log: '',
                    info: '',
                    gasUsed: '23757',
                    gasWanted: '50000',
                    events: [],
                    codespace: '',
                };
                const encoded = encodeToBuffer(
                    ResponseDeliverTx,
                    responseDeliverTx
                );
                assert.equal(
                    encoded.toString('hex').toUpperCase(),
                    '129A030A97030A2A2F7365637265742E636F6D707574652E763162657461312E4D736745786563757465436F6E747261637412E80255CE6C80A3DF474142DAEF33E1E5CCDFD239BABD42202DA5791AA1DC6ED75710420AD19190BF2924CA0EA2922524DE8E729E1BFC9162BFDC8F8483D144112599B81B0C2F67B8ABF6B8B45264E9EA208CC85F250DAE3B848A74F8383538C1D009F86C20B5B9076BC8DC103A944CFFF1A089D2B9E5064279521F10E01A0F519D5689A8979B75D95DAA6E3B8CDAF93F53C38BB8B7631A057668CC63BB9B668E475FC0B8B0D84A4994E1B8AA4B85F4B79FD08522148EB6EC40FB1BB24CD9BCE38DE2378A15DB6048269817C87BA804C4DE8AEF2AC99D68F3C6B495A77C000240E41EEF7AF0613E9D7FE602E3325BB304DF202746FDBDC1BC868108BAB4E0722C870A3F2FF57378F73C49265763D0FF4D37CE1E60882449EF9BDB06CEB129C38CD95FA5EA6DB63742A35596467E927239D24B4103EBEC36004186AB3043BCD4D6B21EBFD20630DB7CE71F66815CA2C83E76BEF17F34AAA9E82CF357595014B0B2F6E86B48C82EDCF36CE128D0860330CDB901'
                );
            });
        });
        describe('decode', () => {
            it('should decode to response deliver tx correctly', () => {
                const message = ResponseDeliverTx.decode(
                    Buffer.from(
                        '129A030A97030A2A2F7365637265742E636F6D707574652E763162657461312E4D736745786563757465436F6E747261637412E80255CE6C80A3DF474142DAEF33E1E5CCDFD239BABD42202DA5791AA1DC6ED75710420AD19190BF2924CA0EA2922524DE8E729E1BFC9162BFDC8F8483D144112599B81B0C2F67B8ABF6B8B45264E9EA208CC85F250DAE3B848A74F8383538C1D009F86C20B5B9076BC8DC103A944CFFF1A089D2B9E5064279521F10E01A0F519D5689A8979B75D95DAA6E3B8CDAF93F53C38BB8B7631A057668CC63BB9B668E475FC0B8B0D84A4994E1B8AA4B85F4B79FD08522148EB6EC40FB1BB24CD9BCE38DE2378A15DB6048269817C87BA804C4DE8AEF2AC99D68F3C6B495A77C000240E41EEF7AF0613E9D7FE602E3325BB304DF202746FDBDC1BC868108BAB4E0722C870A3F2FF57378F73C49265763D0FF4D37CE1E60882449EF9BDB06CEB129C38CD95FA5EA6DB63742A35596467E927239D24B4103EBEC36004186AB3043BCD4D6B21EBFD20630DB7CE71F66815CA2C83E76BEF17F34AAA9E82CF357595014B0B2F6E86B48C82EDCF36CE128D0860330CDB901',
                        'hex'
                    )
                );
                assert.equal(message.code, 0);
                assert.equal(
                    Buffer.from(message.data).toString('base64'),
                    'CpcDCiovc2VjcmV0LmNvbXB1dGUudjFiZXRhMS5Nc2dFeGVjdXRlQ29udHJhY3QS6AJVzmyAo99HQULa7zPh5czf0jm6vUIgLaV5GqHcbtdXEEIK0ZGQvykkyg6ikiUk3o5ynhv8kWK/3I+Eg9FEESWZuBsML2e4q/a4tFJk6eogjMhfJQ2uO4SKdPg4NTjB0An4bCC1uQdryNwQOpRM//GgidK55QZCeVIfEOAaD1GdVomol5t12V2qbjuM2vk/U8OLuLdjGgV2aMxju5tmjkdfwLiw2EpJlOG4qkuF9Lef0IUiFI627ED7G7JM2bzjjeI3ihXbYEgmmBfIe6gExN6K7yrJnWjzxrSVp3wAAkDkHu968GE+nX/mAuMyW7ME3yAnRv29wbyGgQi6tOByLIcKPy/1c3j3PEkmV2PQ/003zh5giCRJ75vbBs6xKcOM2V+l6m22N0KjVZZGfpJyOdJLQQPr7DYAQYarMEO81NayHr/SBjDbfOcfZoFcosg+dr7xfzSqqegs81dZUBSwsvboa0jILtzzbOE='
                );
                assert.equal(message.log, '');
                assert.equal(message.info, '');
                assert.equal(message.gasUsed, '23757');
                assert.equal(message.gasWanted, '50000');
                assert.equal(message.events.length, 0);
                assert.equal(message.codespace, '');
            });
        });
    });
    describe('getSplitPoint', () => {
        it('sanity', () => {
            assert.equal(getSplitPoint(1), 0);
            assert.equal(getSplitPoint(2), 1);
            assert.equal(getSplitPoint(3), 2);
            assert.equal(getSplitPoint(4), 2);
            assert.equal(getSplitPoint(5), 4);
            assert.equal(getSplitPoint(10), 8);
            assert.equal(getSplitPoint(20), 16);
            assert.equal(getSplitPoint(100), 64);
            assert.equal(getSplitPoint(255), 128);
            assert.equal(getSplitPoint(256), 128);
            assert.equal(getSplitPoint(257), 256);
        });
    });
    describe('innerhash', () => {
        it('rfc6962 node inner hash', () => {
            const left = Buffer.from('N123');
            const right = Buffer.from('N456');
            const hash = innerHash(left, right);
            assert.equal(
                hash.toString('hex'),
                'aa217fe888e47007fa15edab33c2b492a722cb106c64667fc2b044444de66bbb'
            );
        });
    });
    describe('leafHash', () => {
        it('rfc6962 empty leaf hash', () => {
            const hash = leafHash(Buffer.from([]));
            assert.equal(
                hash.toString('hex'),
                '6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d'
            );
        });
        describe('leaf hash of buffer', () => {
            const testCases = [
                {
                    data: Buffer.from('1'),
                    hash: '2215e8ac4e2b871c2a48189e79738c956c081e23ac2f2415bf77da199dfd920c',
                },
                {
                    data: Buffer.from('2'),
                    hash: 'fa61e3dec3439589f4784c893bf321d0084f04c572c7af2b68e3f3360a35b486',
                },
                {
                    data: Buffer.from('3'),
                    hash: '906c5d2485cae722073a430f4d04fe1767507592cef226629aeadb85a2ec909d',
                },
                {
                    data: Buffer.from('4'),
                    hash: '11e1f558223f4c71b6be1cecfd1f0de87146d2594877c27b29ec519f9040213c',
                },
            ];
            testCases.forEach((testCase) => {
                it('case: ' + testCase.data, () => {
                    const actual = leafHash(testCase.data);
                    assert.equal(actual.toString('hex'), testCase.hash);
                });
            });
        });
    });
    describe('merkleRoot', () => {
        it('calculates merkle root of 3 leaves', () => {
            const leaves = [
                Buffer.from('1'),
                Buffer.from('2'),
                Buffer.from('3'),
            ];
            const actual = merkleRoot(leaves);
            assert.equal(
                actual.toString('hex'),
                'fe6e9d4604f578602851a2c15ef3894ca07b9517f7d5f7dedc28179ca888580d'
            );
        });
        it('calculates merkle root of 4 leaves', () => {
            const leaves = [
                Buffer.from('1'),
                Buffer.from('2'),
                Buffer.from('3'),
                Buffer.from('4'),
            ];
            const actual = merkleRoot(leaves);
            assert.equal(
                actual.toString('hex'),
                '4c4b77fe3fc6cfb92e4d3c90b5ade42f059a1f112a49827f07edbb7bd4540e7b'
            );
        });
    });
    describe('MerkleProof', () => {
        describe('sanity', () => {
            const testCases = [
                {
                    title: '1 leaves',
                    leaves: [Buffer.from('1')],
                    proofs: [
                        {
                            aunts: [],
                        },
                    ],
                },
                {
                    title: '3 leaves',
                    leaves: [
                        Buffer.from('1'),
                        Buffer.from('2'),
                        Buffer.from('3'),
                    ],
                    proofs: [
                        {
                            aunts: [
                                'fa61e3dec3439589f4784c893bf321d0084f04c572c7af2b68e3f3360a35b486',
                                '906c5d2485cae722073a430f4d04fe1767507592cef226629aeadb85a2ec909d',
                            ],
                        },
                        {
                            aunts: [
                                '2215e8ac4e2b871c2a48189e79738c956c081e23ac2f2415bf77da199dfd920c',
                                '906c5d2485cae722073a430f4d04fe1767507592cef226629aeadb85a2ec909d',
                            ],
                        },
                        {
                            aunts: [
                                'e8bcd97e349693dcfec054fe219ab357b75d3c1cd9f8be1767f6090f9c86f9fd',
                            ],
                        },
                    ],
                },
                {
                    title: '4 leaves',
                    leaves: [
                        Buffer.from('1'),
                        Buffer.from('2'),
                        Buffer.from('3'),
                        Buffer.from('4'),
                    ],
                    proofs: [
                        {
                            aunts: [
                                'fa61e3dec3439589f4784c893bf321d0084f04c572c7af2b68e3f3360a35b486',
                                '9c769ac26f8d61ff40859e5201537845555136f0fd7ab604f7033180fbe76af9',
                            ],
                        },
                        {
                            aunts: [
                                '2215e8ac4e2b871c2a48189e79738c956c081e23ac2f2415bf77da199dfd920c',
                                '9c769ac26f8d61ff40859e5201537845555136f0fd7ab604f7033180fbe76af9',
                            ],
                        },
                        {
                            aunts: [
                                '11e1f558223f4c71b6be1cecfd1f0de87146d2594877c27b29ec519f9040213c',
                                'e8bcd97e349693dcfec054fe219ab357b75d3c1cd9f8be1767f6090f9c86f9fd',
                            ],
                        },
                        {
                            aunts: [
                                '906c5d2485cae722073a430f4d04fe1767507592cef226629aeadb85a2ec909d',
                                'e8bcd97e349693dcfec054fe219ab357b75d3c1cd9f8be1767f6090f9c86f9fd',
                            ],
                        },
                    ],
                },
            ];
            describe('fromLeaves', () => {
                testCases.forEach(({ title, leaves, proofs }) => {
                    describe(title, () => {
                        for (let i = 0; i < leaves.length; i++) {
                            describe('index: ' + i, () => {
                                it('correct merkle proof', () => {
                                    const merkleProof = MerkleProof.fromLeaves(
                                        leaves,
                                        i
                                    );
                                    assert.equal(
                                        merkleProof.total,
                                        leaves.length
                                    );
                                    assert.equal(merkleProof.index, i);
                                    assert.equal(
                                        merkleProof.leaf.toString('hex'),
                                        leaves[i].toString('hex')
                                    );
                                    assert.deepEqual(
                                        merkleProof.aunts.map((aunt) =>
                                            aunt.toString('hex')
                                        ),
                                        proofs[i].aunts
                                    );
                                });
                            });
                        }
                    });
                });
            });
            describe('fromResponseDeliverTxs', () => {
                it('correct merkle proof', () => {
                    const merkleProof = MerkleProof.fromResponseDeliverTxs(
                        [responseDeliverTx],
                        0
                    );
                    assert.equal(merkleProof.total, 1);
                    assert.equal(merkleProof.index, 0);
                    assert.equal(
                        merkleProof.leaf.toString('hex'),
                        encodeToBuffer(
                            ResponseDeliverTx,
                            responseDeliverTx
                        ).toString('hex')
                    );
                    assert.deepEqual(
                        merkleProof.aunts.map((aunt) => aunt.toString('hex')),
                        []
                    );
                });
            });
        });
    });
});
