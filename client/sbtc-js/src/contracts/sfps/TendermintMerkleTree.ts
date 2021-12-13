import { sha256 } from '../../hash';
import * as protobuf from 'protobufjs';
import { Tx, TxResult } from '../../TendermintRPCClient';

// encode tendermint v0.34.0 TxData(ResponseDeliverTx) to leaf of merkle tree in header (last_results_hash)
export class TxDataEncoder {
    public protobufType: protobuf.Type;

    public constructor(protobufType: protobuf.Type) {
        this.protobufType = protobufType;
    }

    public static async init(): Promise<TxDataEncoder> {
        const root = await protobuf.load(__dirname + '/tendermint.abci.proto');
        const protobufType = root.lookupType('ResponseDeliverTx');
        return new TxDataEncoder(protobufType);
    }

    public decode(buffer: Buffer): protobuf.Message {
        return this.protobufType.decode(buffer);
    }

    public encode(tx_result: TxResult): Buffer {
        // set code to undefined if 0(default value)
        let code: number | undefined = tx_result.code;
        if (code == 0) {
            code = undefined;
        }
        const convertedTx = this.protobufType.create({
            code: code,
            data: Buffer.from(tx_result.data, 'base64'),
            gasWanted: parseInt(tx_result.gas_wanted, 10),
            gasUsed: parseInt(tx_result.gas_used, 10),
        });
        const errMsg = this.protobufType.verify(convertedTx);
        if (errMsg) {
            throw Error(errMsg);
        }
        const message = this.protobufType.create(convertedTx);
        return Buffer.from(this.protobufType.encode(message).finish());
    }
}

export class MerkleProof {
    total: number;
    index: number;
    leafHash: Buffer;
    aunts: Buffer[];

    constructor(
        total: number,
        index: number,
        leafHash: Buffer,
        aunts: Buffer[]
    ) {
        this.total = total;
        this.index = index;
        this.leafHash = leafHash;
        this.aunts = aunts;
    }

    public static async fromRpcTxs(
        rpcTxs: Tx[],
        index: number
    ): Promise<MerkleProof> {
        const encoder = await TxDataEncoder.init();
        const leaves = rpcTxs.map((tx) => encoder.encode(tx.tx_result));
        return MerkleProof.fromLeaves(leaves, index);
    }

    public static fromLeaves(leaves: Buffer[], index: number): MerkleProof {
        const total = leaves.length;
        if (index >= total || index < 0) {
            throw new Error('invalid index');
        }
        const aunts = this._buildAunts(leaves, index);
        return new MerkleProof(total, index, leafHash(leaves[index]), aunts);
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
