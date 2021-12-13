// To parse this data:
//
//   import { Convert, HandleMsg, InitMsg, QueryAnswer, QueryMsg } from "./file";
//
//   const handleMsg = Convert.toHandleMsg(json);
//   const initMsg = Convert.toInitMsg(json);
//   const queryAnswer = Convert.toQueryAnswer(json);
//   const queryMsg = Convert.toQueryMsg(json);
//
// These functions will throw an error if the JSON doesn't
// match the expected interface, even if the JSON is valid.

export interface HandleMsg {
    change_finance_admin?: ChangeFinanceAdmin;
    bitcoin_s_p_v_proxy?: BitcoinSPVProxy;
    s_f_p_s_proxy?: SFPSProxy;
}

export interface BitcoinSPVProxy {
    msg: BitcoinSPVHandleMsg;
}

export interface BitcoinSPVHandleMsg {
    add_headers: AddHeaders;
}

export interface AddHeaders {
    headers: string[];
    tip_height: number;
}

export interface ChangeFinanceAdmin {
    new_finance_admin: NewFinanceAdminObject;
}

export interface NewFinanceAdminObject {
    address: string;
    hash: string;
}

export interface SFPSProxy {
    msg: SFPSHandleMsg;
}

export interface SFPSHandleMsg {
    add_light_block?: AddLightBlock;
    add_entropy?: AddEntropy;
}

export interface AddEntropy {
    entropy: string;
}

export interface AddLightBlock {
    current_highest_header: Header;
    light_block: LightBlock;
}

export interface Header {
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
    last_block_id: BlockID;
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
    version?: null | Version;
}

/**
 * Previous block info
 *
 * BlockID
 *
 * Block ID
 */
export interface BlockID {
    hash: string;
    parts?: null | PartSetHeader;
}

/**
 * Block parts header
 */
export interface PartSetHeader {
    hash: string;
    total: number;
}

export interface Version {
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
    header: Header;
}

export interface Commit {
    /**
     * Block ID
     */
    block_id: BlockID;
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

export interface InitMsg {
    config: InitMsgConfig;
}

export interface InitMsgConfig {
    bitcoin_spv: PurpleContractReference;
    finance_admin: PurpleContractReference;
    sfps: PurpleContractReference;
}

export interface PurpleContractReference {
    address: string;
    hash: string;
}

export interface QueryAnswer {
    config: QueryAnswerConfig;
}

export interface QueryAnswerConfig {
    bitcoin_spv: FluffyContractReference;
    finance_admin: FluffyContractReference;
    sfps: FluffyContractReference;
}

export interface FluffyContractReference {
    address: string;
    hash: string;
}

export interface QueryMsg {
    config: { [key: string]: any };
}

// Converts JSON strings to/from your types
// and asserts the results of JSON.parse at runtime
export class Convert {
    public static toHandleMsg(json: string): HandleMsg {
        return cast(JSON.parse(json), r('HandleMsg'));
    }

    public static handleMsgToJson(value: HandleMsg): string {
        return JSON.stringify(uncast(value, r('HandleMsg')), null, 2);
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
    HandleMsg: o(
        [
            {
                json: 'change_finance_admin',
                js: 'change_finance_admin',
                typ: u(undefined, r('ChangeFinanceAdmin')),
            },
            {
                json: 'bitcoin_s_p_v_proxy',
                js: 'bitcoin_s_p_v_proxy',
                typ: u(undefined, r('BitcoinSPVProxy')),
            },
            {
                json: 's_f_p_s_proxy',
                js: 's_f_p_s_proxy',
                typ: u(undefined, r('SFPSProxy')),
            },
        ],
        'any'
    ),
    BitcoinSPVProxy: o(
        [{ json: 'msg', js: 'msg', typ: r('BitcoinSPVHandleMsg') }],
        'any'
    ),
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
    ChangeFinanceAdmin: o(
        [
            {
                json: 'new_finance_admin',
                js: 'new_finance_admin',
                typ: r('NewFinanceAdminObject'),
            },
        ],
        'any'
    ),
    NewFinanceAdminObject: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'hash', js: 'hash', typ: '' },
        ],
        'any'
    ),
    SFPSProxy: o([{ json: 'msg', js: 'msg', typ: r('SFPSHandleMsg') }], 'any'),
    SFPSHandleMsg: o(
        [
            {
                json: 'add_light_block',
                js: 'add_light_block',
                typ: u(undefined, r('AddLightBlock')),
            },
            {
                json: 'add_entropy',
                js: 'add_entropy',
                typ: u(undefined, r('AddEntropy')),
            },
        ],
        'any'
    ),
    AddEntropy: o([{ json: 'entropy', js: 'entropy', typ: '' }], 'any'),
    AddLightBlock: o(
        [
            {
                json: 'current_highest_header',
                js: 'current_highest_header',
                typ: r('Header'),
            },
            { json: 'light_block', js: 'light_block', typ: r('LightBlock') },
        ],
        'any'
    ),
    Header: o(
        [
            { json: 'app_hash', js: 'app_hash', typ: '' },
            { json: 'chain_id', js: 'chain_id', typ: '' },
            { json: 'consensus_hash', js: 'consensus_hash', typ: '' },
            { json: 'data_hash', js: 'data_hash', typ: '' },
            { json: 'evidence_hash', js: 'evidence_hash', typ: '' },
            { json: 'height', js: 'height', typ: '' },
            { json: 'last_block_id', js: 'last_block_id', typ: r('BlockID') },
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
                typ: u(undefined, u(null, r('Version'))),
            },
        ],
        'any'
    ),
    BlockID: o(
        [
            { json: 'hash', js: 'hash', typ: '' },
            {
                json: 'parts',
                js: 'parts',
                typ: u(undefined, u(null, r('PartSetHeader'))),
            },
        ],
        'any'
    ),
    PartSetHeader: o(
        [
            { json: 'hash', js: 'hash', typ: '' },
            { json: 'total', js: 'total', typ: 0 },
        ],
        'any'
    ),
    Version: o(
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
            { json: 'header', js: 'header', typ: r('Header') },
        ],
        'any'
    ),
    Commit: o(
        [
            { json: 'block_id', js: 'block_id', typ: r('BlockID') },
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
    InitMsg: o(
        [{ json: 'config', js: 'config', typ: r('InitMsgConfig') }],
        'any'
    ),
    InitMsgConfig: o(
        [
            {
                json: 'bitcoin_spv',
                js: 'bitcoin_spv',
                typ: r('PurpleContractReference'),
            },
            {
                json: 'finance_admin',
                js: 'finance_admin',
                typ: r('PurpleContractReference'),
            },
            { json: 'sfps', js: 'sfps', typ: r('PurpleContractReference') },
        ],
        'any'
    ),
    PurpleContractReference: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'hash', js: 'hash', typ: '' },
        ],
        'any'
    ),
    QueryAnswer: o(
        [{ json: 'config', js: 'config', typ: r('QueryAnswerConfig') }],
        'any'
    ),
    QueryAnswerConfig: o(
        [
            {
                json: 'bitcoin_spv',
                js: 'bitcoin_spv',
                typ: r('FluffyContractReference'),
            },
            {
                json: 'finance_admin',
                js: 'finance_admin',
                typ: r('FluffyContractReference'),
            },
            { json: 'sfps', js: 'sfps', typ: r('FluffyContractReference') },
        ],
        'any'
    ),
    FluffyContractReference: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'hash', js: 'hash', typ: '' },
        ],
        'any'
    ),
    QueryMsg: o([{ json: 'config', js: 'config', typ: m('any') }], 'any'),
    Type: ['tendermint/PubKeyEd25519'],
};
