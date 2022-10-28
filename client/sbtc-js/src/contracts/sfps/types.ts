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
    config: Config;
    entropy: string;
    initial_header: string;
    max_interval: number;
    seed: number[];
}

export interface Config {
    state_proxy: ContractReference;
}

export interface ContractReference {
    address: string;
    hash: string;
}

export interface QueryAnswer {
    max_interval?: MaxInterval;
    current_highest_header_hash?: CurrentHighestHeaderHash;
    hash_list_length?: HashListLength;
    hash_by_index?: QueryAnswerHashByIndex;
    verify_response_deliver_tx_proof?: QueryAnswerVerifyResponseDeliverTxProof;
    verify_subsequent_light_blocks?: QueryAnswerVerifySubsequentLightBlocks;
}

export interface CurrentHighestHeaderHash {
    hash: string;
    height: number;
}

export interface QueryAnswerHashByIndex {
    hash: string;
    height: number;
}

export interface HashListLength {
    length: number;
}

export interface MaxInterval {
    max_interval: number;
}

export interface QueryAnswerVerifyResponseDeliverTxProof {
    decrypted_data: string;
}

export interface QueryAnswerVerifySubsequentLightBlocks {
    committed_hashes: VerifySubsequentLightBlocksCommittedHashes;
}

export interface VerifySubsequentLightBlocksCommittedHashes {
    commit: number[];
    hashes: PurpleHashes;
}

export interface PurpleHashes {
    anchor_hash: number[];
    anchor_index: number;
    following_hashes: PurpleHeaderHashWithHeight[];
}

export interface PurpleHeaderHashWithHeight {
    hash: number[];
    height: number;
}

export interface QueryMsg {
    max_interval?: { [key: string]: any };
    current_highest_header_hash?: { [key: string]: any };
    hash_list_length?: { [key: string]: any };
    hash_by_index?: QueryMsgHashByIndex;
    verify_response_deliver_tx_proof?: QueryMsgVerifyResponseDeliverTxProof;
    verify_subsequent_light_blocks?: QueryMsgVerifySubsequentLightBlocks;
}

export interface QueryMsgHashByIndex {
    index: number;
}

export interface QueryMsgVerifyResponseDeliverTxProof {
    block_hash_index: number;
    encryption_key: string;
    headers: string[];
    merkle_proof: MerkleProof;
}

export interface MerkleProof {
    aunts: string[];
    index: number;
    leaf: string;
    total: number;
}

export interface QueryMsgVerifySubsequentLightBlocks {
    anchor_header: string;
    anchor_header_index: number;
    commit_flags: boolean[];
    following_light_blocks: string[];
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
    anchor_hash: number[];
    anchor_index: number;
    following_hashes: FluffyHeaderHashWithHeight[];
}

export interface FluffyHeaderHashWithHeight {
    hash: number[];
    height: number;
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
            { json: 'config', js: 'config', typ: r('Config') },
            { json: 'entropy', js: 'entropy', typ: '' },
            { json: 'initial_header', js: 'initial_header', typ: '' },
            { json: 'max_interval', js: 'max_interval', typ: 0 },
            { json: 'seed', js: 'seed', typ: a(0) },
        ],
        'any'
    ),
    Config: o(
        [
            {
                json: 'state_proxy',
                js: 'state_proxy',
                typ: r('ContractReference'),
            },
        ],
        'any'
    ),
    ContractReference: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'hash', js: 'hash', typ: '' },
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
                json: 'verify_response_deliver_tx_proof',
                js: 'verify_response_deliver_tx_proof',
                typ: u(undefined, r('QueryAnswerVerifyResponseDeliverTxProof')),
            },
            {
                json: 'verify_subsequent_light_blocks',
                js: 'verify_subsequent_light_blocks',
                typ: u(undefined, r('QueryAnswerVerifySubsequentLightBlocks')),
            },
        ],
        'any'
    ),
    CurrentHighestHeaderHash: o(
        [
            { json: 'hash', js: 'hash', typ: '' },
            { json: 'height', js: 'height', typ: 0 },
        ],
        'any'
    ),
    QueryAnswerHashByIndex: o(
        [
            { json: 'hash', js: 'hash', typ: '' },
            { json: 'height', js: 'height', typ: 0 },
        ],
        'any'
    ),
    HashListLength: o([{ json: 'length', js: 'length', typ: 0 }], 'any'),
    MaxInterval: o(
        [{ json: 'max_interval', js: 'max_interval', typ: 0 }],
        'any'
    ),
    QueryAnswerVerifyResponseDeliverTxProof: o(
        [{ json: 'decrypted_data', js: 'decrypted_data', typ: '' }],
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
            { json: 'anchor_hash', js: 'anchor_hash', typ: a(0) },
            { json: 'anchor_index', js: 'anchor_index', typ: 0 },
            {
                json: 'following_hashes',
                js: 'following_hashes',
                typ: a(r('PurpleHeaderHashWithHeight')),
            },
        ],
        'any'
    ),
    PurpleHeaderHashWithHeight: o(
        [
            { json: 'hash', js: 'hash', typ: a(0) },
            { json: 'height', js: 'height', typ: 0 },
        ],
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
                json: 'verify_response_deliver_tx_proof',
                js: 'verify_response_deliver_tx_proof',
                typ: u(undefined, r('QueryMsgVerifyResponseDeliverTxProof')),
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
    QueryMsgVerifyResponseDeliverTxProof: o(
        [
            { json: 'block_hash_index', js: 'block_hash_index', typ: 0 },
            { json: 'encryption_key', js: 'encryption_key', typ: '' },
            { json: 'headers', js: 'headers', typ: a('') },
            { json: 'merkle_proof', js: 'merkle_proof', typ: r('MerkleProof') },
        ],
        'any'
    ),
    MerkleProof: o(
        [
            { json: 'aunts', js: 'aunts', typ: a('') },
            { json: 'index', js: 'index', typ: 0 },
            { json: 'leaf', js: 'leaf', typ: '' },
            { json: 'total', js: 'total', typ: 0 },
        ],
        'any'
    ),
    QueryMsgVerifySubsequentLightBlocks: o(
        [
            { json: 'anchor_header', js: 'anchor_header', typ: '' },
            { json: 'anchor_header_index', js: 'anchor_header_index', typ: 0 },
            { json: 'commit_flags', js: 'commit_flags', typ: a(true) },
            {
                json: 'following_light_blocks',
                js: 'following_light_blocks',
                typ: a(''),
            },
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
            { json: 'anchor_hash', js: 'anchor_hash', typ: a(0) },
            { json: 'anchor_index', js: 'anchor_index', typ: 0 },
            {
                json: 'following_hashes',
                js: 'following_hashes',
                typ: a(r('FluffyHeaderHashWithHeight')),
            },
        ],
        'any'
    ),
    FluffyHeaderHashWithHeight: o(
        [
            { json: 'hash', js: 'hash', typ: a(0) },
            { json: 'height', js: 'height', typ: 0 },
        ],
        'any'
    ),
};
