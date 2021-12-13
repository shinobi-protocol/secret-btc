import {
    MerkleProof,
    MerkleTree,
} from '../src/contracts/bitcoin_spv/BitcoinMerkleTree';
import { assert } from 'chai';
import { describe, it } from 'mocha';

describe('MerkleTree', () => {
    describe('build', () => {
        describe('should build merkle tree', () => {
            const testCases = [
                {
                    title: '4 leaves',
                    leaves: [
                        '37221d338269b6d12ad29a20e4beb9506526dded90eadb89e6074d231ac4d1f6',
                        '03fbad67c14af86d280218ed971a98c7d14fe7f10417c1f350403b411f97a9dc',
                        '7af61424f4f131892d6e972bec2d599b84affeafd958a7e7a8530aa0f1004790',
                        '829b2a5ad886e897cc7a71da7d89519b11b0c474e650238ffb0f5163bac588cf',
                    ],
                    expected: [
                        [
                            '37221d338269b6d12ad29a20e4beb9506526dded90eadb89e6074d231ac4d1f6',
                            '03fbad67c14af86d280218ed971a98c7d14fe7f10417c1f350403b411f97a9dc',
                            '7af61424f4f131892d6e972bec2d599b84affeafd958a7e7a8530aa0f1004790',
                            '829b2a5ad886e897cc7a71da7d89519b11b0c474e650238ffb0f5163bac588cf',
                        ],
                        [
                            '265144e841c7866949f8b2795a44161fb771dcf06345dacd74b880a57a7a4b01',
                            'bca5faae4e577c6ba65e951a800c1c7409a7903aaedd173e51bcd54804fcffa9',
                        ],
                        [
                            '32c7b4757d148e39aabf4ff74d3a898effbef3240ec948d23cac448cbd5c457b',
                        ],
                    ],
                },
                {
                    title: '5 leaves',
                    leaves: [
                        'cafebeef',
                        'ffffffff',
                        'aaaaaaaa',
                        'bbbbbbbb',
                        'cccccccc',
                    ],
                    expected: [
                        [
                            'cafebeef',
                            'ffffffff',
                            'aaaaaaaa',
                            'bbbbbbbb',
                            'cccccccc',
                        ],
                        [
                            '00ffb46c84f38500f566ffd235caed6e61ecb37793184a08d9f077e5cc9c63d4',
                            'ce85c79cfb3af65a6416762a5b47ecb9071bfcf41531d9aca5e4c277125a5fbc',
                            '5dee81eeeaeaeb470f9d15670ed68237b2ee7b309e6d1ad1ed4339f59fc6e19b',
                        ],
                        [
                            '8fa1c2b671bba5eb667894218b3cf2855d8e0e85c91dc0b0b899b7cc8b0670e8',
                            '3b275abb86323974940fcdda044ef8e310a48d2135228cc03d89fb3c5c51507c',
                        ],
                        [
                            '6624bd5c857daa7202c7b3eaa4285dab60595116b28f1ab4212ef480d79ef667',
                        ],
                    ],
                },
            ];
            testCases.forEach((testCase) => {
                it('case: ' + testCase.title, () => {
                    const leaves = testCase.leaves.map((x) => {
                        return Buffer.from(x, 'hex');
                    });
                    const merkleTree = MerkleTree.build(leaves);
                    const expected = new MerkleTree(
                        testCase.expected.map((level) => {
                            return level.map((x) => {
                                return Buffer.from(x, 'hex');
                            });
                        })
                    );
                    assert.deepEqual(merkleTree, expected);
                });
            });
        });
    });
    describe('merkleProof', () => {
        describe('should build merkle proof', () => {
            const testCases = [
                {
                    title: '4 leaves',
                    leaves: [
                        '37221d338269b6d12ad29a20e4beb9506526dded90eadb89e6074d231ac4d1f6',
                        '03fbad67c14af86d280218ed971a98c7d14fe7f10417c1f350403b411f97a9dc',
                        '7af61424f4f131892d6e972bec2d599b84affeafd958a7e7a8530aa0f1004790',
                        '829b2a5ad886e897cc7a71da7d89519b11b0c474e650238ffb0f5163bac588cf',
                    ],
                    proofs: [
                        {
                            leaf: 0,
                            proof: {
                                siblings: [
                                    [0, 0],
                                    [0, 1],
                                    [1, 1],
                                ],
                                prefix: [false, false],
                            },
                        },
                        {
                            leaf: 1,
                            proof: {
                                siblings: [
                                    [0, 1],
                                    [0, 0],
                                    [1, 1],
                                ],
                                prefix: [true, false],
                            },
                        },
                        {
                            leaf: 2,
                            proof: {
                                siblings: [
                                    [0, 2],
                                    [0, 3],
                                    [1, 0],
                                ],
                                prefix: [false, true],
                            },
                        },
                        {
                            leaf: 3,
                            proof: {
                                siblings: [
                                    [0, 3],
                                    [0, 2],
                                    [1, 0],
                                ],
                                prefix: [true, true],
                            },
                        },
                    ],
                },
                {
                    title: '5 leaves',
                    leaves: [
                        'cafebeef',
                        'ffffffff',
                        'aaaaaaaa',
                        'bbbbbbbb',
                        'cccccccc',
                    ],
                    proofs: [
                        {
                            leaf: 0,
                            proof: {
                                siblings: [
                                    [0, 0],
                                    [0, 1],
                                    [1, 1],
                                    [2, 1],
                                ],
                                prefix: [false, false, false],
                            },
                        },
                        {
                            leaf: 1,
                            proof: {
                                siblings: [
                                    [0, 1],
                                    [0, 0],
                                    [1, 1],
                                    [2, 1],
                                ],
                                prefix: [true, false, false],
                            },
                        },
                        {
                            leaf: 2,
                            proof: {
                                siblings: [
                                    [0, 2],
                                    [0, 3],
                                    [1, 0],
                                    [2, 1],
                                ],
                                prefix: [false, true, false],
                            },
                        },
                        {
                            leaf: 3,
                            proof: {
                                siblings: [
                                    [0, 3],
                                    [0, 2],
                                    [1, 0],
                                    [2, 1],
                                ],
                                prefix: [true, true, false],
                            },
                        },
                        {
                            leaf: 4,
                            proof: {
                                siblings: [
                                    [0, 4],
                                    [0, 4],
                                    [1, 2],
                                    [2, 0],
                                ],
                                prefix: [false, false, true],
                            },
                        },
                    ],
                },
            ];
            testCases.forEach((testCase) => {
                describe('case: ' + testCase.title, () => {
                    const merkleTree = MerkleTree.build(
                        testCase.leaves.map((x) => {
                            return Buffer.from(x, 'hex');
                        })
                    );
                    const tree = merkleTree.tree;
                    testCase.proofs.forEach((proof, i) => {
                        it('pattern: ' + (i + 1).toString(), () => {
                            const expectedPath = new MerkleProof(
                                proof.proof.siblings.map((sibling) => {
                                    return tree[sibling[0]][sibling[1]];
                                }),
                                proof.proof.prefix
                            );
                            const merkleProof = merkleTree.merkleProof(
                                tree[0][proof.leaf]
                            );
                            assert.deepEqual(merkleProof, expectedPath);
                            assert.deepEqual(
                                merkleTree.root(),
                                merkleProof.calcMerkleRoot()
                            );
                        });
                    });
                });
            });
        });
    });
});
