import { sha256 } from '../../hash';
import { ResponseDeliverTx } from 'secretjs/dist/protobuf_stuff/tendermint/abci/types';
import { encodeToBuffer } from '../../proto';

export class MerkleProof {
    total: number;
    index: number;
    aunts: Buffer[];
    leaf: Buffer;

    constructor(total: number, index: number, aunts: Buffer[], leaf: Buffer) {
        this.total = total;
        this.index = index;
        this.aunts = aunts;
        this.leaf = leaf;
    }

    public static fromResponseDeliverTxs(
        responseDeliverTxs: ResponseDeliverTx[],
        index: number
    ): MerkleProof {
        const leaves = responseDeliverTxs.map((tx) =>
            encodeResponseDeliverTxToMerkleLeaf(tx)
        );
        return MerkleProof.fromLeaves(leaves, index);
    }

    public static fromLeaves(leaves: Buffer[], index: number): MerkleProof {
        const total = leaves.length;
        if (index >= total || index < 0) {
            throw new Error('invalid index');
        }
        const aunts = this._buildAunts(leaves, index);
        return new MerkleProof(total, index, aunts, leaves[index]);
    }

    private static _buildAunts(leaves: Buffer[], index: number): Buffer[] {
        if (leaves.length === 1) {
            return [];
        }
        const splitPoint = getSplitPoint(leaves.length);
        if (index < splitPoint) {
            // if target leaf is on left of split point
            const aunts = this._buildAunts(leaves.slice(0, splitPoint), index);
            const rightAunt = merkleRoot(leaves.slice(splitPoint));
            aunts.push(rightAunt);
            return aunts;
        } else {
            // if target leaf is on right of split point
            const aunts = this._buildAunts(
                leaves.slice(splitPoint),
                index - splitPoint
            );
            const leftAunt = merkleRoot(leaves.slice(0, splitPoint));
            aunts.push(leftAunt);
            return aunts;
        }
    }
}

export function getSplitPoint(n: number): number {
    if (n < 1) {
        throw Error('Trying to split tree with length < 1');
    }
    if (n === 1) {
        return 0;
    }

    let mid = 2 ** Math.floor(Math.log2(n));
    if (mid === n) {
        mid /= 2;
    }
    return mid;
}

export function leafHash(leaf: Buffer): Buffer {
    return sha256(Buffer.concat([Buffer.from([0]), leaf]));
}

export function innerHash(left: Buffer, right: Buffer): Buffer {
    return sha256(Buffer.concat([Buffer.from([1]), left, right]));
}

export function merkleRoot(leaves: Buffer[]): Buffer {
    const length = leaves.length;
    if (length === 0) {
        // empty hash
        return leafHash(Buffer.from([0]));
    }
    if (length === 1) {
        // leaf hash
        return leafHash(leaves[0]);
    }
    const splitPoint = getSplitPoint(length);
    const left = merkleRoot(leaves.slice(0, splitPoint));
    const right = merkleRoot(leaves.slice(splitPoint));
    // inner hash
    return innerHash(left, right);
}

function encodeResponseDeliverTxToMerkleLeaf(tx: ResponseDeliverTx): Buffer {
    if (tx.log !== '' || tx.info !== '' || tx.events.length !== 0) {
        throw new Error(
            'ResponseDeliverTx contains nondeterministic fields. log, info, events are must be zero values.' +
                JSON.stringify(tx)
        );
    }
    return encodeToBuffer(ResponseDeliverTx, tx);
}
