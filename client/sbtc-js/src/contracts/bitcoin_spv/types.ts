// To parse this data:
//
//   import { Convert, BitcoinSPVHandleMsg, InitMsg, QueryAnswer, QueryMsg } from "./file";
//
//   const bitcoinSPVHandleMsg = Convert.toBitcoinSPVHandleMsg(json);
//   const initMsg = Convert.toInitMsg(json);
//   const queryAnswer = Convert.toQueryAnswer(json);
//   const queryMsg = Convert.toQueryMsg(json);
//
// These functions will throw an error if the JSON doesn't
// match the expected interface, even if the JSON is valid.

export interface BitcoinSPVHandleMsg {
    add_headers: AddHeaders;
}

export interface AddHeaders {
    headers: string[];
    tip_height: number;
}

export interface InitMsg {
    bitcoin_network: string;
    confirmation: number;
    initial_header?: null | InitialHeader;
    seed: number[];
    state_proxy: InitMsgStateProxy;
}

export interface InitialHeader {
    header: string;
    height: number;
}

export interface InitMsgStateProxy {
    address: string;
    hash: string;
}

export interface QueryAnswer {
    block_header?: QueryAnswerBlockHeader;
    best_header_hash?: BestHeaderHash;
    config?: Config;
    verify_merkle_proof?: QueryAnswerVerifyMerkleProof;
}

export interface BestHeaderHash {
    hash: string;
}

export interface QueryAnswerBlockHeader {
    header: string;
}

/**
 * Contract Config set at contrat init.
 */
export interface Config {
    /**
     * "bitcoin" | "testnet" | 'regtest"
     */
    bitcoin_network: string;
    /**
     * minimum block needed for tx confirmed
     */
    confirmation: number;
    state_proxy: ConfigStateProxy;
}

export interface ConfigStateProxy {
    address: string;
    hash: string;
}

export interface QueryAnswerVerifyMerkleProof {
    success: boolean;
}

export interface QueryMsg {
    block_header?: QueryMsgBlockHeader;
    best_header_hash?: { [key: string]: any };
    verify_merkle_proof?: QueryMsgVerifyMerkleProof;
    config?: { [key: string]: any };
}

export interface QueryMsgBlockHeader {
    height: number;
}

export interface QueryMsgVerifyMerkleProof {
    height: number;
    merkle_proof: MerkleProofMsg;
    tx: string;
}

export interface MerkleProofMsg {
    prefix: boolean[];
    siblings: string[];
}

// Converts JSON strings to/from your types
// and asserts the results of JSON.parse at runtime
export class Convert {
    public static toBitcoinSPVHandleMsg(json: string): BitcoinSPVHandleMsg {
        return cast(JSON.parse(json), r('BitcoinSPVHandleMsg'));
    }

    public static bitcoinSPVHandleMsgToJson(
        value: BitcoinSPVHandleMsg
    ): string {
        return JSON.stringify(uncast(value, r('BitcoinSPVHandleMsg')), null, 2);
    }

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
    BitcoinSPVHandleMsg: o(
        [{ json: 'add_headers', js: 'add_headers', typ: r('AddHeaders') }],
        'any'
    ),
    AddHeaders: o(
        [
            { json: 'headers', js: 'headers', typ: a('') },
            { json: 'tip_height', js: 'tip_height', typ: 0 },
        ],
        'any'
    ),
    InitMsg: o(
        [
            { json: 'bitcoin_network', js: 'bitcoin_network', typ: '' },
            { json: 'confirmation', js: 'confirmation', typ: 0 },
            {
                json: 'initial_header',
                js: 'initial_header',
                typ: u(undefined, u(null, r('InitialHeader'))),
            },
            { json: 'seed', js: 'seed', typ: a(0) },
            {
                json: 'state_proxy',
                js: 'state_proxy',
                typ: r('InitMsgStateProxy'),
            },
        ],
        'any'
    ),
    InitialHeader: o(
        [
            { json: 'header', js: 'header', typ: '' },
            { json: 'height', js: 'height', typ: 0 },
        ],
        'any'
    ),
    InitMsgStateProxy: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'hash', js: 'hash', typ: '' },
        ],
        'any'
    ),
    QueryAnswer: o(
        [
            {
                json: 'block_header',
                js: 'block_header',
                typ: u(undefined, r('QueryAnswerBlockHeader')),
            },
            {
                json: 'best_header_hash',
                js: 'best_header_hash',
                typ: u(undefined, r('BestHeaderHash')),
            },
            { json: 'config', js: 'config', typ: u(undefined, r('Config')) },
            {
                json: 'verify_merkle_proof',
                js: 'verify_merkle_proof',
                typ: u(undefined, r('QueryAnswerVerifyMerkleProof')),
            },
        ],
        'any'
    ),
    BestHeaderHash: o([{ json: 'hash', js: 'hash', typ: '' }], 'any'),
    QueryAnswerBlockHeader: o(
        [{ json: 'header', js: 'header', typ: '' }],
        'any'
    ),
    Config: o(
        [
            { json: 'bitcoin_network', js: 'bitcoin_network', typ: '' },
            { json: 'confirmation', js: 'confirmation', typ: 0 },
            {
                json: 'state_proxy',
                js: 'state_proxy',
                typ: r('ConfigStateProxy'),
            },
        ],
        'any'
    ),
    ConfigStateProxy: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'hash', js: 'hash', typ: '' },
        ],
        'any'
    ),
    QueryAnswerVerifyMerkleProof: o(
        [{ json: 'success', js: 'success', typ: true }],
        'any'
    ),
    QueryMsg: o(
        [
            {
                json: 'block_header',
                js: 'block_header',
                typ: u(undefined, r('QueryMsgBlockHeader')),
            },
            {
                json: 'best_header_hash',
                js: 'best_header_hash',
                typ: u(undefined, m('any')),
            },
            {
                json: 'verify_merkle_proof',
                js: 'verify_merkle_proof',
                typ: u(undefined, r('QueryMsgVerifyMerkleProof')),
            },
            { json: 'config', js: 'config', typ: u(undefined, m('any')) },
        ],
        'any'
    ),
    QueryMsgBlockHeader: o([{ json: 'height', js: 'height', typ: 0 }], 'any'),
    QueryMsgVerifyMerkleProof: o(
        [
            { json: 'height', js: 'height', typ: 0 },
            {
                json: 'merkle_proof',
                js: 'merkle_proof',
                typ: r('MerkleProofMsg'),
            },
            { json: 'tx', js: 'tx', typ: '' },
        ],
        'any'
    ),
    MerkleProofMsg: o(
        [
            { json: 'prefix', js: 'prefix', typ: a(true) },
            { json: 'siblings', js: 'siblings', typ: a('') },
        ],
        'any'
    ),
};
