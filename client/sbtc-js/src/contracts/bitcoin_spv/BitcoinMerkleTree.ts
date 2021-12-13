/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import { Transaction } from 'bitcoinjs-lib';
import { sha256d } from '../../hash';
import { MerkleProofMsg } from './types';

export class MerkleProof {
    siblings: Buffer[];
    prefix: boolean[];
    constructor(siblings: Buffer[], prefix: boolean[]) {
        if (siblings.length === 0) {
            throw new Error('No siblings');
        }
        if (siblings.length - 1 != prefix.length) {
            throw new Error('Invalid prefix');
        }
        this.siblings = siblings;
        this.prefix = prefix;
    }

    public calcMerkleRoot(): Buffer {
        let current = this.siblings[0];
        for (let i = 0; i < this.prefix.length; i++) {
            const sibling = this.siblings[i + 1];
            const prefix = this.prefix[i];
            if (prefix) {
                current = sha256d(Buffer.concat([sibling, current]));
            } else {
                current = sha256d(Buffer.concat([current, sibling]));
            }
        }
        return current;
    }

    public leaf(): Buffer {
        return this.siblings[0];
    }

    public encodeToContractMsg(): MerkleProofMsg {
        return {
            prefix: this.prefix,
            siblings: this.siblings.map((sibling) => {
                const copy = Buffer.alloc(32);
                sibling.copy(copy);
                return copy.reverse().toString('hex');
            }),
        };
    }
}

export class TreePosition {
    public height: number;
    public pos: number;
    constructor(height: number, pos: number) {
        if (!Number.isInteger(height) && height >= 0) {
            throw new Error('height is not unsigned integer');
        }
        if (!Number.isInteger(pos) && pos >= 0) {
            throw new Error('pos is not unsigned integer');
        }
        this.height = height;
        this.pos = pos;
    }

    public parent(): TreePosition {
        return new TreePosition(this.height + 1, Math.floor(this.pos / 2));
    }
}

export class MerkleTree {
    public tree: Buffer[][];
    constructor(tree: Buffer[][]) {
        this.tree = tree;
    }

    public static fromTxs(txs: Transaction[]): MerkleTree {
        return this.build(
            txs.map((tx) => {
                return tx.getHash();
            })
        );
    }

    public static build(leaves: Buffer[]): MerkleTree {
        return new MerkleTree(this._build(leaves));
    }

    private static _build(leaves: Buffer[]): Buffer[][] {
        const length = leaves.length;
        if (length === 1) {
            return [leaves];
        }
        const results = [];
        for (let i = 0; i < length; i += 2) {
            const left = leaves[i];
            const right = i + 1 === length ? left : leaves[i + 1];
            const data = Buffer.concat([left, right]);
            results.push(sha256d(data));
        }
        return [leaves].concat(this._build(results));
    }

    public root(): Buffer {
        return this.tree[this.tree.length - 1][0];
    }

    public hashAt(position: TreePosition): Buffer {
        if (
            this.treeHeight() <= position.height ||
            this.lengthAt(position.height) <= position.pos
        ) {
            throw new Error('position is out of tree');
        }
        return this.tree[position.height][position.pos];
    }

    public treeHeight(): number {
        return this.tree.length;
    }

    public lengthAt(height: number): number {
        return this.tree[height].length;
    }

    public getPosition(hash: Buffer): TreePosition | undefined {
        for (let i = 0; i < this.tree.length; i++) {
            for (let j = 0; j < this.tree[i].length; j++) {
                if (this.tree[i][j].equals(hash)) {
                    return new TreePosition(i, j);
                }
            }
        }
        return undefined;
    }

    public merkleProof(leaf: Buffer): MerkleProof {
        const leafPos = this.getPosition(leaf);
        if (leafPos === undefined) {
            throw new Error('invalid leaf');
        }
        const siblings: Buffer[] = [];
        const prefix: boolean[] = [];
        siblings.push(this.hashAt(leafPos));
        this._buildMerkleProof(leafPos, siblings, prefix);
        return new MerkleProof(siblings, prefix);
    }

    public equals(other: MerkleTree): boolean {
        if (this.tree.length !== other.tree.length) {
            return false;
        }
        for (let i = 0; i < this.tree.length; i++) {
            if (this.tree[i].length !== other.tree[i].length) {
                return false;
            }
            for (let j = 0; j < this.tree[i].length; j++) {
                if (!this.tree[i][j].equals(other.tree[i][j])) {
                    return false;
                }
            }
        }
        return true;
    }

    private _buildMerkleProof(
        pos: TreePosition,
        siblings: Buffer[],
        prefix: boolean[]
    ) {
        if (pos.height === this.treeHeight() - 1) {
            return;
        }
        const even = pos.pos % 2 === 1;
        if (even) {
            siblings.push(
                this.hashAt(new TreePosition(pos.height, pos.pos - 1))
            );
        } else {
            let sibling;
            if (pos.pos === this.lengthAt(pos.height) - 1) {
                sibling = this.hashAt(pos);
            } else {
                sibling = this.hashAt(
                    new TreePosition(pos.height, pos.pos + 1)
                );
            }
            siblings.push(sibling);
        }
        prefix.push(even);
        this._buildMerkleProof(pos.parent(), siblings, prefix);
    }
}
