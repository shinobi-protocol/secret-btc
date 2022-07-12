// To parse this data:
//
//   import { Convert, InitMsg, QueryAnswer, QueryMsg, SFPSHandleMsg } from "./file";
//
//   const initMsg = Convert.toInitMsg(json);
//   const queryAnswer = Convert.toQueryAnswer(json);
//   const queryMsg = Convert.toQueryMsg(json);
//   const sFPSHandleMsg = Convert.toSFPSHandleMsg(json);
//
// These functions will throw an error if the JSON doesn't
// match the expected interface, even if the JSON is valid.

export interface InitMsg {
    entropy: string;
    initial_header: InitialHeaderObject;
    max_interval: number;
}

export interface InitialHeaderObject {
    /**
     * State after txs from the previous block
     */
    app_hash: string;
    /**
     * Chain ID
     */
    chain_id: string;
    /**
     * Consensus params for the current block
     */
    consensus_hash: string;
    /**
     * Merkle root of transaction hashes
     */
    data_hash: string;
    /**
     * Hash of evidence included in the block
     */
    evidence_hash: string;
    /**
     * Current block height
     */
    height: string;
    /**
     * Previous block info
     */
    last_block_id: InitialHeaderLastBlockID;
    /**
     * Commit from validators from the last block
     */
    last_commit_hash: string;
    /**
     * Root hash of all results from the txs from the previous block
     */
    last_results_hash: string;
    /**
     * Validators for the next block
     */
    next_validators_hash: string;
    /**
     * Original proposer of the block
     */
    proposer_address: string;
    /**
     * Current timestamp
     */
    time: string;
    /**
     * Validators for the current block
     */
    validators_hash: string;
    /**
     * Header version
     */
    version?: null | InitialHeaderVersion;
}

/**
 * Previous block info
 *
 * BlockID
 */
export interface InitialHeaderLastBlockID {
    hash: string;
    parts?: null | PurplePartSetHeader;
}

/**
 * Block parts header
 */
export interface PurplePartSetHeader {
    hash: string;
    total: number;
}

export interface InitialHeaderVersion {
    /**
     * App version
     */
    app?: string;
    /**
     * Block version
     */
    block: string;
}

export interface QueryAnswer {
    max_interval?: MaxInterval;
    current_highest_header_hash?: CurrentHighestHeaderHash;
    hash_list_length?: HashListLength;
    hash_by_index?: QueryAnswerHashByIndex;
    verify_tx_result_proof?: QueryAnswerVerifyTxResultProof;
    verify_subsequent_light_blocks?: QueryAnswerVerifySubsequentLightBlocks;
}

export interface CurrentHighestHeaderHash {
    hash: string;
}

export interface QueryAnswerHashByIndex {
    hash: string;
}

export interface HashListLength {
    length: number;
}

export interface MaxInterval {
    max_interval: number;
}

export interface QueryAnswerVerifySubsequentLightBlocks {
    committed_hashes: VerifySubsequentLightBlocksCommittedHashes;
}

export interface VerifySubsequentLightBlocksCommittedHashes {
    commit: number[];
    hashes: PurpleHashes;
}

export interface PurpleHashes {
    first_hash: number[];
    following_hashes: Array<number[]>;
}

export interface QueryAnswerVerifyTxResultProof {
    decrypted_data: string;
}

export interface QueryMsg {
    max_interval?: { [key: string]: any };
    current_highest_header_hash?: { [key: string]: any };
    hash_list_length?: { [key: string]: any };
    hash_by_index?: QueryMsgHashByIndex;
    verify_tx_result_proof?: QueryMsgVerifyTxResultProof;
    verify_subsequent_light_blocks?: QueryMsgVerifySubsequentLightBlocks;
}

export interface QueryMsgHashByIndex {
    index: number;
}

export interface QueryMsgVerifySubsequentLightBlocks {
    current_highest_header: CurrentHighestHeaderElement;
    light_blocks: LightBlock[];
}

export interface CurrentHighestHeaderElement {
    /**
     * State after txs from the previous block
     */
    app_hash: string;
    /**
     * Chain ID
     */
    chain_id: string;
    /**
     * Consensus params for the current block
     */
    consensus_hash: string;
    /**
     * Merkle root of transaction hashes
     */
    data_hash: string;
    /**
     * Hash of evidence included in the block
     */
    evidence_hash: string;
    /**
     * Current block height
     */
    height: string;
    /**
     * Previous block info
     */
    last_block_id: BlockIDObject;
    /**
     * Commit from validators from the last block
     */
    last_commit_hash: string;
    /**
     * Root hash of all results from the txs from the previous block
     */
    last_results_hash: string;
    /**
     * Validators for the next block
     */
    next_validators_hash: string;
    /**
     * Original proposer of the block
     */
    proposer_address: string;
    /**
     * Current timestamp
     */
    time: string;
    /**
     * Validators for the current block
     */
    validators_hash: string;
    /**
     * Header version
     */
    version?: null | CurrentHighestHeaderVersion;
}

/**
 * Previous block info
 *
 * BlockID
 *
 * Block ID
 */
export interface BlockIDObject {
    hash: string;
    parts?: null | BlockIDPartSetHeader;
}

/**
 * Block parts header
 */
export interface BlockIDPartSetHeader {
    hash: string;
    total: number;
}

export interface CurrentHighestHeaderVersion {
    /**
     * App version
     */
    app?: string;
    /**
     * Block version
     */
    block: string;
}

export interface LightBlock {
    signed_header: SignedHeader;
    validators: ValidatorInfo[];
}

export interface SignedHeader {
    commit: Commit;
    header: CurrentHighestHeaderElement;
}

export interface Commit {
    /**
     * Block ID
     */
    block_id: BlockIDObject;
    /**
     * Block height
     */
    height: string;
    /**
     * Round
     */
    round: number;
    /**
     * Votes
     */
    signatures: CommitSig[];
}

export interface CommitSig {
    block_id_flag: number;
    signature?: null | string;
    timestamp: string;
    validator_address: string;
}

export interface ValidatorInfo {
    address: string;
    pub_key: PublicKey;
    voting_power: string;
}

export interface PublicKey {
    type: Type;
    value: string;
}

export enum Type {
    TendermintPubKeyEd25519 = 'tendermint/PubKeyEd25519',
}

export interface QueryMsgVerifyTxResultProof {
    encryption_key: string;
    header_hash_index: number;
    tx_result_proof: TxResultProof;
}

export interface TxResultProof {
    headers: CurrentHighestHeaderElement[];
    merkle_proof: MerkleProof;
    tx_result: TxResult;
}

export interface MerkleProof {
    aunts: string[];
    index: number;
    leaf_hash: string;
    total: number;
}

export interface TxResult {
    code: number;
    data: string;
    gas_used: string;
    gas_wanted: string;
}

export interface SFPSHandleMsg {
    append_subsequent_hashes: AppendSubsequentHashes;
}

export interface AppendSubsequentHashes {
    committed_hashes: AppendSubsequentHashesCommittedHashes;
}

export interface AppendSubsequentHashesCommittedHashes {
    commit: number[];
    hashes: FluffyHashes;
}

export interface FluffyHashes {
    first_hash: number[];
    following_hashes: Array<number[]>;
}

// Converts JSON strings to/from your types
// and asserts the results of JSON.parse at runtime
export class Convert {
    public static toInitMsg(json: string): InitMsg {
        return cast(JSON.parse(json), r('InitMsg'));
    }

    public static initMsgToJson(value: InitMsg): string {
        return JSON.stringify(uncast(value, r('InitMsg')), null, 2);
    }

    public static toQueryAnswer(json: string): QueryAnswer {
        return cast(JSON.parse(json), r('QueryAnswer'));
    }

    public static queryAnswerToJson(value: QueryAnswer): string {
        return JSON.stringify(uncast(value, r('QueryAnswer')), null, 2);
    }

    public static toQueryMsg(json: string): QueryMsg {
        return cast(JSON.parse(json), r('QueryMsg'));
    }

    public static queryMsgToJson(value: QueryMsg): string {
        return JSON.stringify(uncast(value, r('QueryMsg')), null, 2);
    }

    public static toSFPSHandleMsg(json: string): SFPSHandleMsg {
        return cast(JSON.parse(json), r('SFPSHandleMsg'));
    }

    public static sFPSHandleMsgToJson(value: SFPSHandleMsg): string {
        return JSON.stringify(uncast(value, r('SFPSHandleMsg')), null, 2);
    }
}

function invalidValue(typ: any, val: any, key: any = ''): never {
    if (key) {
        throw Error(
            `Invalid value for key "${key}". Expected type ${JSON.stringify(
                typ
            )} but got ${JSON.stringify(val)}`
        );
    }
    throw Error(
        `Invalid value ${JSON.stringify(val)} for type ${JSON.stringify(typ)}`
    );
}

function jsonToJSProps(typ: any): any {
    if (typ.jsonToJS === undefined) {
        const map: any = {};
        typ.props.forEach(
            (p: any) => (map[p.json] = { key: p.js, typ: p.typ })
        );
        typ.jsonToJS = map;
    }
    return typ.jsonToJS;
}

function jsToJSONProps(typ: any): any {
    if (typ.jsToJSON === undefined) {
        const map: any = {};
        typ.props.forEach(
            (p: any) => (map[p.js] = { key: p.json, typ: p.typ })
        );
        typ.jsToJSON = map;
    }
    return typ.jsToJSON;
}

function transform(val: any, typ: any, getProps: any, key: any = ''): any {
    function transformPrimitive(typ: string, val: any): any {
        if (typeof typ === typeof val) return val;
        return invalidValue(typ, val, key);
    }

    function transformUnion(typs: any[], val: any): any {
        // val must validate against one typ in typs
        const l = typs.length;
        for (let i = 0; i < l; i++) {
            const typ = typs[i];
            try {
                return transform(val, typ, getProps);
            } catch (_) {}
        }
        return invalidValue(typs, val);
    }

    function transformEnum(cases: string[], val: any): any {
        if (cases.indexOf(val) !== -1) return val;
        return invalidValue(cases, val);
    }

    function transformArray(typ: any, val: any): any {
        // val must be an array with no invalid elements
        if (!Array.isArray(val)) return invalidValue('array', val);
        return val.map((el) => transform(el, typ, getProps));
    }

    function transformDate(val: any): any {
        if (val === null) {
            return null;
        }
        const d = new Date(val);
        if (isNaN(d.valueOf())) {
            return invalidValue('Date', val);
        }
        return d;
    }

    function transformObject(
        props: { [k: string]: any },
        additional: any,
        val: any
    ): any {
        if (val === null || typeof val !== 'object' || Array.isArray(val)) {
            return invalidValue('object', val);
        }
        const result: any = {};
        Object.getOwnPropertyNames(props).forEach((key) => {
            const prop = props[key];
            const v = Object.prototype.hasOwnProperty.call(val, key)
                ? val[key]
                : undefined;
            result[prop.key] = transform(v, prop.typ, getProps, prop.key);
        });
        Object.getOwnPropertyNames(val).forEach((key) => {
            if (!Object.prototype.hasOwnProperty.call(props, key)) {
                result[key] = transform(val[key], additional, getProps, key);
            }
        });
        return result;
    }

    if (typ === 'any') return val;
    if (typ === null) {
        if (val === null) return val;
        return invalidValue(typ, val);
    }
    if (typ === false) return invalidValue(typ, val);
    while (typeof typ === 'object' && typ.ref !== undefined) {
        typ = typeMap[typ.ref];
    }
    if (Array.isArray(typ)) return transformEnum(typ, val);
    if (typeof typ === 'object') {
        return typ.hasOwnProperty('unionMembers')
            ? transformUnion(typ.unionMembers, val)
            : typ.hasOwnProperty('arrayItems')
            ? transformArray(typ.arrayItems, val)
            : typ.hasOwnProperty('props')
            ? transformObject(getProps(typ), typ.additional, val)
            : invalidValue(typ, val);
    }
    // Numbers can be parsed by Date but shouldn't be.
    if (typ === Date && typeof val !== 'number') return transformDate(val);
    return transformPrimitive(typ, val);
}

function cast<T>(val: any, typ: any): T {
    return transform(val, typ, jsonToJSProps);
}

function uncast<T>(val: T, typ: any): any {
    return transform(val, typ, jsToJSONProps);
}

function a(typ: any) {
    return { arrayItems: typ };
}

function u(...typs: any[]) {
    return { unionMembers: typs };
}

function o(props: any[], additional: any) {
    return { props, additional };
}

function m(additional: any) {
    return { props: [], additional };
}

function r(name: string) {
    return { ref: name };
}

const typeMap: any = {
    InitMsg: o(
        [
            { json: 'entropy', js: 'entropy', typ: '' },
            {
                json: 'initial_header',
                js: 'initial_header',
                typ: r('InitialHeaderObject'),
            },
            { json: 'max_interval', js: 'max_interval', typ: 0 },
        ],
        'any'
    ),
    InitialHeaderObject: o(
        [
            { json: 'app_hash', js: 'app_hash', typ: '' },
            { json: 'chain_id', js: 'chain_id', typ: '' },
            { json: 'consensus_hash', js: 'consensus_hash', typ: '' },
            { json: 'data_hash', js: 'data_hash', typ: '' },
            { json: 'evidence_hash', js: 'evidence_hash', typ: '' },
            { json: 'height', js: 'height', typ: '' },
            {
                json: 'last_block_id',
                js: 'last_block_id',
                typ: r('InitialHeaderLastBlockID'),
            },
            { json: 'last_commit_hash', js: 'last_commit_hash', typ: '' },
            { json: 'last_results_hash', js: 'last_results_hash', typ: '' },
            {
                json: 'next_validators_hash',
                js: 'next_validators_hash',
                typ: '',
            },
            { json: 'proposer_address', js: 'proposer_address', typ: '' },
            { json: 'time', js: 'time', typ: '' },
            { json: 'validators_hash', js: 'validators_hash', typ: '' },
            {
                json: 'version',
                js: 'version',
                typ: u(undefined, u(null, r('InitialHeaderVersion'))),
            },
        ],
        'any'
    ),
    InitialHeaderLastBlockID: o(
        [
            { json: 'hash', js: 'hash', typ: '' },
            {
                json: 'parts',
                js: 'parts',
                typ: u(undefined, u(null, r('PurplePartSetHeader'))),
            },
        ],
        'any'
    ),
    PurplePartSetHeader: o(
        [
            { json: 'hash', js: 'hash', typ: '' },
            { json: 'total', js: 'total', typ: 0 },
        ],
        'any'
    ),
    InitialHeaderVersion: o(
        [
            { json: 'app', js: 'app', typ: u(undefined, '') },
            { json: 'block', js: 'block', typ: '' },
        ],
        'any'
    ),
    QueryAnswer: o(
        [
            {
                json: 'max_interval',
                js: 'max_interval',
                typ: u(undefined, r('MaxInterval')),
            },
            {
                json: 'current_highest_header_hash',
                js: 'current_highest_header_hash',
                typ: u(undefined, r('CurrentHighestHeaderHash')),
            },
            {
                json: 'hash_list_length',
                js: 'hash_list_length',
                typ: u(undefined, r('HashListLength')),
            },
            {
                json: 'hash_by_index',
                js: 'hash_by_index',
                typ: u(undefined, r('QueryAnswerHashByIndex')),
            },
            {
                json: 'verify_tx_result_proof',
                js: 'verify_tx_result_proof',
                typ: u(undefined, r('QueryAnswerVerifyTxResultProof')),
            },
            {
                json: 'verify_subsequent_light_blocks',
                js: 'verify_subsequent_light_blocks',
                typ: u(undefined, r('QueryAnswerVerifySubsequentLightBlocks')),
            },
        ],
        'any'
    ),
    CurrentHighestHeaderHash: o([{ json: 'hash', js: 'hash', typ: '' }], 'any'),
    QueryAnswerHashByIndex: o([{ json: 'hash', js: 'hash', typ: '' }], 'any'),
    HashListLength: o([{ json: 'length', js: 'length', typ: 0 }], 'any'),
    MaxInterval: o(
        [{ json: 'max_interval', js: 'max_interval', typ: 0 }],
        'any'
    ),
    QueryAnswerVerifySubsequentLightBlocks: o(
        [
            {
                json: 'committed_hashes',
                js: 'committed_hashes',
                typ: r('VerifySubsequentLightBlocksCommittedHashes'),
            },
        ],
        'any'
    ),
    VerifySubsequentLightBlocksCommittedHashes: o(
        [
            { json: 'commit', js: 'commit', typ: a(0) },
            { json: 'hashes', js: 'hashes', typ: r('PurpleHashes') },
        ],
        'any'
    ),
    PurpleHashes: o(
        [
            { json: 'first_hash', js: 'first_hash', typ: a(0) },
            { json: 'following_hashes', js: 'following_hashes', typ: a(a(0)) },
        ],
        'any'
    ),
    QueryAnswerVerifyTxResultProof: o(
        [{ json: 'decrypted_data', js: 'decrypted_data', typ: '' }],
        'any'
    ),
    QueryMsg: o(
        [
            {
                json: 'max_interval',
                js: 'max_interval',
                typ: u(undefined, m('any')),
            },
            {
                json: 'current_highest_header_hash',
                js: 'current_highest_header_hash',
                typ: u(undefined, m('any')),
            },
            {
                json: 'hash_list_length',
                js: 'hash_list_length',
                typ: u(undefined, m('any')),
            },
            {
                json: 'hash_by_index',
                js: 'hash_by_index',
                typ: u(undefined, r('QueryMsgHashByIndex')),
            },
            {
                json: 'verify_tx_result_proof',
                js: 'verify_tx_result_proof',
                typ: u(undefined, r('QueryMsgVerifyTxResultProof')),
            },
            {
                json: 'verify_subsequent_light_blocks',
                js: 'verify_subsequent_light_blocks',
                typ: u(undefined, r('QueryMsgVerifySubsequentLightBlocks')),
            },
        ],
        'any'
    ),
    QueryMsgHashByIndex: o([{ json: 'index', js: 'index', typ: 0 }], 'any'),
    QueryMsgVerifySubsequentLightBlocks: o(
        [
            {
                json: 'current_highest_header',
                js: 'current_highest_header',
                typ: r('CurrentHighestHeaderElement'),
            },
            {
                json: 'light_blocks',
                js: 'light_blocks',
                typ: a(r('LightBlock')),
            },
        ],
        'any'
    ),
    CurrentHighestHeaderElement: o(
        [
            { json: 'app_hash', js: 'app_hash', typ: '' },
            { json: 'chain_id', js: 'chain_id', typ: '' },
            { json: 'consensus_hash', js: 'consensus_hash', typ: '' },
            { json: 'data_hash', js: 'data_hash', typ: '' },
            { json: 'evidence_hash', js: 'evidence_hash', typ: '' },
            { json: 'height', js: 'height', typ: '' },
            {
                json: 'last_block_id',
                js: 'last_block_id',
                typ: r('BlockIDObject'),
            },
            { json: 'last_commit_hash', js: 'last_commit_hash', typ: '' },
            { json: 'last_results_hash', js: 'last_results_hash', typ: '' },
            {
                json: 'next_validators_hash',
                js: 'next_validators_hash',
                typ: '',
            },
            { json: 'proposer_address', js: 'proposer_address', typ: '' },
            { json: 'time', js: 'time', typ: '' },
            { json: 'validators_hash', js: 'validators_hash', typ: '' },
            {
                json: 'version',
                js: 'version',
                typ: u(undefined, u(null, r('CurrentHighestHeaderVersion'))),
            },
        ],
        'any'
    ),
    BlockIDObject: o(
        [
            { json: 'hash', js: 'hash', typ: '' },
            {
                json: 'parts',
                js: 'parts',
                typ: u(undefined, u(null, r('BlockIDPartSetHeader'))),
            },
        ],
        'any'
    ),
    BlockIDPartSetHeader: o(
        [
            { json: 'hash', js: 'hash', typ: '' },
            { json: 'total', js: 'total', typ: 0 },
        ],
        'any'
    ),
    CurrentHighestHeaderVersion: o(
        [
            { json: 'app', js: 'app', typ: u(undefined, '') },
            { json: 'block', js: 'block', typ: '' },
        ],
        'any'
    ),
    LightBlock: o(
        [
            {
                json: 'signed_header',
                js: 'signed_header',
                typ: r('SignedHeader'),
            },
            {
                json: 'validators',
                js: 'validators',
                typ: a(r('ValidatorInfo')),
            },
        ],
        'any'
    ),
    SignedHeader: o(
        [
            { json: 'commit', js: 'commit', typ: r('Commit') },
            {
                json: 'header',
                js: 'header',
                typ: r('CurrentHighestHeaderElement'),
            },
        ],
        'any'
    ),
    Commit: o(
        [
            { json: 'block_id', js: 'block_id', typ: r('BlockIDObject') },
            { json: 'height', js: 'height', typ: '' },
            { json: 'round', js: 'round', typ: 0 },
            { json: 'signatures', js: 'signatures', typ: a(r('CommitSig')) },
        ],
        'any'
    ),
    CommitSig: o(
        [
            { json: 'block_id_flag', js: 'block_id_flag', typ: 0 },
            {
                json: 'signature',
                js: 'signature',
                typ: u(undefined, u(null, '')),
            },
            { json: 'timestamp', js: 'timestamp', typ: '' },
            { json: 'validator_address', js: 'validator_address', typ: '' },
        ],
        'any'
    ),
    ValidatorInfo: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'pub_key', js: 'pub_key', typ: r('PublicKey') },
            { json: 'voting_power', js: 'voting_power', typ: '' },
        ],
        'any'
    ),
    PublicKey: o(
        [
            { json: 'type', js: 'type', typ: r('Type') },
            { json: 'value', js: 'value', typ: '' },
        ],
        'any'
    ),
    QueryMsgVerifyTxResultProof: o(
        [
            { json: 'encryption_key', js: 'encryption_key', typ: '' },
            { json: 'header_hash_index', js: 'header_hash_index', typ: 0 },
            {
                json: 'tx_result_proof',
                js: 'tx_result_proof',
                typ: r('TxResultProof'),
            },
        ],
        'any'
    ),
    TxResultProof: o(
        [
            {
                json: 'headers',
                js: 'headers',
                typ: a(r('CurrentHighestHeaderElement')),
            },
            { json: 'merkle_proof', js: 'merkle_proof', typ: r('MerkleProof') },
            { json: 'tx_result', js: 'tx_result', typ: r('TxResult') },
        ],
        'any'
    ),
    MerkleProof: o(
        [
            { json: 'aunts', js: 'aunts', typ: a('') },
            { json: 'index', js: 'index', typ: 0 },
            { json: 'leaf_hash', js: 'leaf_hash', typ: '' },
            { json: 'total', js: 'total', typ: 0 },
        ],
        'any'
    ),
    TxResult: o(
        [
            { json: 'code', js: 'code', typ: 0 },
            { json: 'data', js: 'data', typ: '' },
            { json: 'gas_used', js: 'gas_used', typ: '' },
            { json: 'gas_wanted', js: 'gas_wanted', typ: '' },
        ],
        'any'
    ),
    SFPSHandleMsg: o(
        [
            {
                json: 'append_subsequent_hashes',
                js: 'append_subsequent_hashes',
                typ: r('AppendSubsequentHashes'),
            },
        ],
        'any'
    ),
    AppendSubsequentHashes: o(
        [
            {
                json: 'committed_hashes',
                js: 'committed_hashes',
                typ: r('AppendSubsequentHashesCommittedHashes'),
            },
        ],
        'any'
    ),
    AppendSubsequentHashesCommittedHashes: o(
        [
            { json: 'commit', js: 'commit', typ: a(0) },
            { json: 'hashes', js: 'hashes', typ: r('FluffyHashes') },
        ],
        'any'
    ),
    FluffyHashes: o(
        [
            { json: 'first_hash', js: 'first_hash', typ: a(0) },
            { json: 'following_hashes', js: 'following_hashes', typ: a(a(0)) },
        ],
        'any'
    ),
    Type: ['tendermint/PubKeyEd25519'],
};
