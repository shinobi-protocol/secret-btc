// To parse this data:
//
//   import { Convert, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg } from "./file";
//
//   const handleAnswer = Convert.toHandleAnswer(json);
//   const handleMsg = Convert.toHandleMsg(json);
//   const initMsg = Convert.toInitMsg(json);
//   const queryAnswer = Convert.toQueryAnswer(json);
//   const queryMsg = Convert.toQueryMsg(json);
//
// These functions will throw an error if the JSON doesn't
// match the expected interface, even if the JSON is valid.

export interface HandleAnswer {
    create_viewing_key?: HandleAnswerCreateViewingKey;
    request_mint_address?: HandleAnswerRequestMintAddress;
    release_incorrect_amount_b_t_c?: HandleAnswerReleaseIncorrectAmountBTC;
    claim_released_btc?: HandleAnswerClaimReleasedBtc;
    request_release_btc?: HandleAnswerRequestReleaseBtc;
    release_btc_by_owner?: HandleAnswerReleaseBtcByOwner;
}

export interface HandleAnswerClaimReleasedBtc {
    tx: string;
}

export interface HandleAnswerCreateViewingKey {
    key: string;
}

export interface HandleAnswerReleaseBtcByOwner {
    tx: string;
}

export interface HandleAnswerReleaseIncorrectAmountBTC {
    tx: string;
}

export interface HandleAnswerRequestMintAddress {
    mint_address: string;
}

export interface HandleAnswerRequestReleaseBtc {
    request_key: number[];
}

export interface HandleMsg {
    create_viewing_key?: HandleMsgCreateViewingKey;
    set_viewing_key?: SetViewingKey;
    request_mint_address?: HandleMsgRequestMintAddress;
    verify_mint_tx?: VerifyMintTx;
    release_incorrect_amount_b_t_c?: HandleMsgReleaseIncorrectAmountBTC;
    request_release_btc?: HandleMsgRequestReleaseBtc;
    claim_released_btc?: HandleMsgClaimReleasedBtc;
    change_owner?: ChangeOwner;
    set_suspension_switch?: SetSuspensionSwitch;
    release_btc_by_owner?: HandleMsgReleaseBtcByOwner;
}

export interface ChangeOwner {
    new_owner: string;
}

export interface HandleMsgClaimReleasedBtc {
    block_hash_index: number;
    encryption_key: string;
    fee_per_vb: number;
    headers: string[];
    merkle_proof: MerkleProof;
    recipient_address: string;
}

export interface MerkleProof {
    aunts: string[];
    index: number;
    leaf: string;
    total: number;
}

export interface HandleMsgCreateViewingKey {
    entropy: string;
}

export interface HandleMsgReleaseBtcByOwner {
    fee_per_vb: number;
    max_input_length: number;
    recipient_address: string;
    tx_value: number;
}

export interface HandleMsgReleaseIncorrectAmountBTC {
    fee_per_vb: number;
    height: number;
    merkle_proof: MerkleProofMsg;
    recipient_address: string;
    tx: string;
}

export interface MerkleProofMsg {
    prefix: boolean[];
    siblings: string[];
}

export interface HandleMsgRequestMintAddress {
    entropy: string;
}

export interface HandleMsgRequestReleaseBtc {
    amount: number;
    entropy: string;
}

export interface SetSuspensionSwitch {
    suspension_switch: SetSuspensionSwitchSuspensionSwitch;
}

export interface SetSuspensionSwitchSuspensionSwitch {
    claim_release_btc: boolean;
    release_incorrect_amount_btc: boolean;
    request_mint_address: boolean;
    request_release_btc: boolean;
    verify_mint_tx: boolean;
}

export interface SetViewingKey {
    key: string;
}

export interface VerifyMintTx {
    height: number;
    merkle_proof: MerkleProofMsg;
    tx: string;
}

export interface InitMsg {
    config: InitMsgConfig;
    seed: number[];
}

export interface InitMsgConfig {
    /**
     * [Contract References]
     */
    bitcoin_spv: PurpleContractReference;
    /**
     * [Bitcoin] Unit of utxo value that the contrat accepts
     */
    btc_tx_values: number[];
    log: PurpleContractReference;
    /**
     * [Owner]
     */
    owner: string;
    sbtc: PurpleContractReference;
    sfps: PurpleContractReference;
    state_proxy: PurpleContractReference;
}

/**
 * [Contract References]
 */
export interface PurpleContractReference {
    address: string;
    hash: string;
}

export interface QueryAnswer {
    mint_address?: QueryAnswerMintAddress;
    config?: QueryAnswerConfig;
    suspension_switch?: QueryAnswerSuspensionSwitch;
    viewing_key_error?: ViewingKeyError;
}

export interface QueryAnswerConfig {
    /**
     * [Contract References]
     */
    bitcoin_spv: FluffyContractReference;
    /**
     * [Bitcoin] Unit of utxo value that the contrat accepts
     */
    btc_tx_values: number[];
    log: FluffyContractReference;
    /**
     * [Owner]
     */
    owner: string;
    sbtc: FluffyContractReference;
    sfps: FluffyContractReference;
    state_proxy: FluffyContractReference;
}

/**
 * [Contract References]
 */
export interface FluffyContractReference {
    address: string;
    hash: string;
}

export interface QueryAnswerMintAddress {
    address?: null | string;
}

export interface QueryAnswerSuspensionSwitch {
    claim_release_btc: boolean;
    release_incorrect_amount_btc: boolean;
    request_mint_address: boolean;
    request_release_btc: boolean;
    verify_mint_tx: boolean;
}

export interface ViewingKeyError {
    msg: string;
}

export interface QueryMsg {
    mint_address?: QueryMsgMintAddress;
    suspension_switch?: { [key: string]: any };
    config?: { [key: string]: any };
}

export interface QueryMsgMintAddress {
    address: string;
    key: string;
}

// Converts JSON strings to/from your types
// and asserts the results of JSON.parse at runtime
export class Convert {
    public static toHandleAnswer(json: string): HandleAnswer {
        return cast(JSON.parse(json), r('HandleAnswer'));
    }

    public static handleAnswerToJson(value: HandleAnswer): string {
        return JSON.stringify(uncast(value, r('HandleAnswer')), null, 2);
    }

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
    HandleAnswer: o(
        [
            {
                json: 'create_viewing_key',
                js: 'create_viewing_key',
                typ: u(undefined, r('HandleAnswerCreateViewingKey')),
            },
            {
                json: 'request_mint_address',
                js: 'request_mint_address',
                typ: u(undefined, r('HandleAnswerRequestMintAddress')),
            },
            {
                json: 'release_incorrect_amount_b_t_c',
                js: 'release_incorrect_amount_b_t_c',
                typ: u(undefined, r('HandleAnswerReleaseIncorrectAmountBTC')),
            },
            {
                json: 'claim_released_btc',
                js: 'claim_released_btc',
                typ: u(undefined, r('HandleAnswerClaimReleasedBtc')),
            },
            {
                json: 'request_release_btc',
                js: 'request_release_btc',
                typ: u(undefined, r('HandleAnswerRequestReleaseBtc')),
            },
            {
                json: 'release_btc_by_owner',
                js: 'release_btc_by_owner',
                typ: u(undefined, r('HandleAnswerReleaseBtcByOwner')),
            },
        ],
        'any'
    ),
    HandleAnswerClaimReleasedBtc: o([{ json: 'tx', js: 'tx', typ: '' }], 'any'),
    HandleAnswerCreateViewingKey: o(
        [{ json: 'key', js: 'key', typ: '' }],
        'any'
    ),
    HandleAnswerReleaseBtcByOwner: o(
        [{ json: 'tx', js: 'tx', typ: '' }],
        'any'
    ),
    HandleAnswerReleaseIncorrectAmountBTC: o(
        [{ json: 'tx', js: 'tx', typ: '' }],
        'any'
    ),
    HandleAnswerRequestMintAddress: o(
        [{ json: 'mint_address', js: 'mint_address', typ: '' }],
        'any'
    ),
    HandleAnswerRequestReleaseBtc: o(
        [{ json: 'request_key', js: 'request_key', typ: a(0) }],
        'any'
    ),
    HandleMsg: o(
        [
            {
                json: 'create_viewing_key',
                js: 'create_viewing_key',
                typ: u(undefined, r('HandleMsgCreateViewingKey')),
            },
            {
                json: 'set_viewing_key',
                js: 'set_viewing_key',
                typ: u(undefined, r('SetViewingKey')),
            },
            {
                json: 'request_mint_address',
                js: 'request_mint_address',
                typ: u(undefined, r('HandleMsgRequestMintAddress')),
            },
            {
                json: 'verify_mint_tx',
                js: 'verify_mint_tx',
                typ: u(undefined, r('VerifyMintTx')),
            },
            {
                json: 'release_incorrect_amount_b_t_c',
                js: 'release_incorrect_amount_b_t_c',
                typ: u(undefined, r('HandleMsgReleaseIncorrectAmountBTC')),
            },
            {
                json: 'request_release_btc',
                js: 'request_release_btc',
                typ: u(undefined, r('HandleMsgRequestReleaseBtc')),
            },
            {
                json: 'claim_released_btc',
                js: 'claim_released_btc',
                typ: u(undefined, r('HandleMsgClaimReleasedBtc')),
            },
            {
                json: 'change_owner',
                js: 'change_owner',
                typ: u(undefined, r('ChangeOwner')),
            },
            {
                json: 'set_suspension_switch',
                js: 'set_suspension_switch',
                typ: u(undefined, r('SetSuspensionSwitch')),
            },
            {
                json: 'release_btc_by_owner',
                js: 'release_btc_by_owner',
                typ: u(undefined, r('HandleMsgReleaseBtcByOwner')),
            },
        ],
        'any'
    ),
    ChangeOwner: o([{ json: 'new_owner', js: 'new_owner', typ: '' }], 'any'),
    HandleMsgClaimReleasedBtc: o(
        [
            { json: 'block_hash_index', js: 'block_hash_index', typ: 0 },
            { json: 'encryption_key', js: 'encryption_key', typ: '' },
            { json: 'fee_per_vb', js: 'fee_per_vb', typ: 0 },
            { json: 'headers', js: 'headers', typ: a('') },
            { json: 'merkle_proof', js: 'merkle_proof', typ: r('MerkleProof') },
            { json: 'recipient_address', js: 'recipient_address', typ: '' },
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
    HandleMsgCreateViewingKey: o(
        [{ json: 'entropy', js: 'entropy', typ: '' }],
        'any'
    ),
    HandleMsgReleaseBtcByOwner: o(
        [
            { json: 'fee_per_vb', js: 'fee_per_vb', typ: 0 },
            { json: 'max_input_length', js: 'max_input_length', typ: 0 },
            { json: 'recipient_address', js: 'recipient_address', typ: '' },
            { json: 'tx_value', js: 'tx_value', typ: 0 },
        ],
        'any'
    ),
    HandleMsgReleaseIncorrectAmountBTC: o(
        [
            { json: 'fee_per_vb', js: 'fee_per_vb', typ: 0 },
            { json: 'height', js: 'height', typ: 0 },
            {
                json: 'merkle_proof',
                js: 'merkle_proof',
                typ: r('MerkleProofMsg'),
            },
            { json: 'recipient_address', js: 'recipient_address', typ: '' },
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
    HandleMsgRequestMintAddress: o(
        [{ json: 'entropy', js: 'entropy', typ: '' }],
        'any'
    ),
    HandleMsgRequestReleaseBtc: o(
        [
            { json: 'amount', js: 'amount', typ: 0 },
            { json: 'entropy', js: 'entropy', typ: '' },
        ],
        'any'
    ),
    SetSuspensionSwitch: o(
        [
            {
                json: 'suspension_switch',
                js: 'suspension_switch',
                typ: r('SetSuspensionSwitchSuspensionSwitch'),
            },
        ],
        'any'
    ),
    SetSuspensionSwitchSuspensionSwitch: o(
        [
            { json: 'claim_release_btc', js: 'claim_release_btc', typ: true },
            {
                json: 'release_incorrect_amount_btc',
                js: 'release_incorrect_amount_btc',
                typ: true,
            },
            {
                json: 'request_mint_address',
                js: 'request_mint_address',
                typ: true,
            },
            {
                json: 'request_release_btc',
                js: 'request_release_btc',
                typ: true,
            },
            { json: 'verify_mint_tx', js: 'verify_mint_tx', typ: true },
        ],
        'any'
    ),
    SetViewingKey: o([{ json: 'key', js: 'key', typ: '' }], 'any'),
    VerifyMintTx: o(
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
    InitMsg: o(
        [
            { json: 'config', js: 'config', typ: r('InitMsgConfig') },
            { json: 'seed', js: 'seed', typ: a(0) },
        ],
        'any'
    ),
    InitMsgConfig: o(
        [
            {
                json: 'bitcoin_spv',
                js: 'bitcoin_spv',
                typ: r('PurpleContractReference'),
            },
            { json: 'btc_tx_values', js: 'btc_tx_values', typ: a(0) },
            { json: 'log', js: 'log', typ: r('PurpleContractReference') },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'sbtc', js: 'sbtc', typ: r('PurpleContractReference') },
            { json: 'sfps', js: 'sfps', typ: r('PurpleContractReference') },
            {
                json: 'state_proxy',
                js: 'state_proxy',
                typ: r('PurpleContractReference'),
            },
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
        [
            {
                json: 'mint_address',
                js: 'mint_address',
                typ: u(undefined, r('QueryAnswerMintAddress')),
            },
            {
                json: 'config',
                js: 'config',
                typ: u(undefined, r('QueryAnswerConfig')),
            },
            {
                json: 'suspension_switch',
                js: 'suspension_switch',
                typ: u(undefined, r('QueryAnswerSuspensionSwitch')),
            },
            {
                json: 'viewing_key_error',
                js: 'viewing_key_error',
                typ: u(undefined, r('ViewingKeyError')),
            },
        ],
        'any'
    ),
    QueryAnswerConfig: o(
        [
            {
                json: 'bitcoin_spv',
                js: 'bitcoin_spv',
                typ: r('FluffyContractReference'),
            },
            { json: 'btc_tx_values', js: 'btc_tx_values', typ: a(0) },
            { json: 'log', js: 'log', typ: r('FluffyContractReference') },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'sbtc', js: 'sbtc', typ: r('FluffyContractReference') },
            { json: 'sfps', js: 'sfps', typ: r('FluffyContractReference') },
            {
                json: 'state_proxy',
                js: 'state_proxy',
                typ: r('FluffyContractReference'),
            },
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
    QueryAnswerMintAddress: o(
        [{ json: 'address', js: 'address', typ: u(undefined, u(null, '')) }],
        'any'
    ),
    QueryAnswerSuspensionSwitch: o(
        [
            { json: 'claim_release_btc', js: 'claim_release_btc', typ: true },
            {
                json: 'release_incorrect_amount_btc',
                js: 'release_incorrect_amount_btc',
                typ: true,
            },
            {
                json: 'request_mint_address',
                js: 'request_mint_address',
                typ: true,
            },
            {
                json: 'request_release_btc',
                js: 'request_release_btc',
                typ: true,
            },
            { json: 'verify_mint_tx', js: 'verify_mint_tx', typ: true },
        ],
        'any'
    ),
    ViewingKeyError: o([{ json: 'msg', js: 'msg', typ: '' }], 'any'),
    QueryMsg: o(
        [
            {
                json: 'mint_address',
                js: 'mint_address',
                typ: u(undefined, r('QueryMsgMintAddress')),
            },
            {
                json: 'suspension_switch',
                js: 'suspension_switch',
                typ: u(undefined, m('any')),
            },
            { json: 'config', js: 'config', typ: u(undefined, m('any')) },
        ],
        'any'
    ),
    QueryMsgMintAddress: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'key', js: 'key', typ: '' },
        ],
        'any'
    ),
};
