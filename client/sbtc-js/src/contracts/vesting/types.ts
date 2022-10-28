// To parse this data:
//
//   import { Convert, HandleAnswer, HandleMsg, LockMsg, QueryAnswer, QueryMsg } from "./file";
//
//   const handleAnswer = Convert.toHandleAnswer(json);
//   const handleMsg = Convert.toHandleMsg(json);
//   const lockMsg = Convert.toLockMsg(json);
//   const queryAnswer = Convert.toQueryAnswer(json);
//   const queryMsg = Convert.toQueryMsg(json);
//
// These functions will throw an error if the JSON doesn't
// match the expected interface, even if the JSON is valid.

export interface HandleAnswer {
    receive?: ReceiveObject;
    unlock?: Unlock;
}

export interface ReceiveObject {
    claimed_amount: string;
    end_time: number;
    id: number;
    locked_amount: string;
    locker: string;
    recipient: string;
    remaining_amount: string;
    start_time: number;
    token: ReceiveToken;
}

export interface ReceiveToken {
    address: string;
    hash: string;
}

export interface Unlock {
    id: number;
}

export interface HandleMsg {
    receive?: Snip20ReceiveMsg;
    claim?: Claim;
}

export interface Claim {
    id: number;
}

/**
 * Snip20ReceiveMsg should be de/serialized under `Receive()` variant in a HandleMsg
 */
export interface Snip20ReceiveMsg {
    amount: string;
    from: string;
    memo?: null | string;
    msg?: null | string;
    sender: string;
}

export interface LockMsg {
    contract_hash: string;
    end_time: number;
    recipient: string;
}

export interface QueryAnswer {
    latest_i_d?: number;
    vesting_infos?: AccountInfoElement[];
    account_infos?: AccountInfoElement[];
    vesting_summary?: VestingSummary;
}

export interface AccountInfoElement {
    claimed_amount: string;
    end_time: number;
    id: number;
    locked_amount: string;
    locker: string;
    recipient: string;
    remaining_amount: string;
    start_time: number;
    token: AccountInfoToken;
}

export interface AccountInfoToken {
    address: string;
    hash: string;
}

export interface VestingSummary {
    total_claimed: string;
    total_locked: string;
    total_remaining: string;
}

export interface QueryMsg {
    latest_i_d?: { [key: string]: any };
    vesting_infos?: VestingInfos;
    recipients_vesting_infos?: RecipientsVestingInfos;
    vesting_summary?: QueryMsgVestingSummary;
}

export interface RecipientsVestingInfos {
    page: number;
    page_size: number;
    recipient: string;
}

export interface VestingInfos {
    ids: number[];
}

export interface QueryMsgVestingSummary {
    token: string;
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

    public static toLockMsg(json: string): LockMsg {
        return cast(JSON.parse(json), r('LockMsg'));
    }

    public static lockMsgToJson(value: LockMsg): string {
        return JSON.stringify(uncast(value, r('LockMsg')), null, 2);
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
                json: 'receive',
                js: 'receive',
                typ: u(undefined, r('ReceiveObject')),
            },
            { json: 'unlock', js: 'unlock', typ: u(undefined, r('Unlock')) },
        ],
        'any'
    ),
    ReceiveObject: o(
        [
            { json: 'claimed_amount', js: 'claimed_amount', typ: '' },
            { json: 'end_time', js: 'end_time', typ: 0 },
            { json: 'id', js: 'id', typ: 0 },
            { json: 'locked_amount', js: 'locked_amount', typ: '' },
            { json: 'locker', js: 'locker', typ: '' },
            { json: 'recipient', js: 'recipient', typ: '' },
            { json: 'remaining_amount', js: 'remaining_amount', typ: '' },
            { json: 'start_time', js: 'start_time', typ: 0 },
            { json: 'token', js: 'token', typ: r('ReceiveToken') },
        ],
        'any'
    ),
    ReceiveToken: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'hash', js: 'hash', typ: '' },
        ],
        'any'
    ),
    Unlock: o([{ json: 'id', js: 'id', typ: 0 }], 'any'),
    HandleMsg: o(
        [
            {
                json: 'receive',
                js: 'receive',
                typ: u(undefined, r('Snip20ReceiveMsg')),
            },
            { json: 'claim', js: 'claim', typ: u(undefined, r('Claim')) },
        ],
        'any'
    ),
    Claim: o([{ json: 'id', js: 'id', typ: 0 }], 'any'),
    Snip20ReceiveMsg: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'from', js: 'from', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'msg', js: 'msg', typ: u(undefined, u(null, '')) },
            { json: 'sender', js: 'sender', typ: '' },
        ],
        'any'
    ),
    LockMsg: o(
        [
            { json: 'contract_hash', js: 'contract_hash', typ: '' },
            { json: 'end_time', js: 'end_time', typ: 0 },
            { json: 'recipient', js: 'recipient', typ: '' },
        ],
        'any'
    ),
    QueryAnswer: o(
        [
            { json: 'latest_i_d', js: 'latest_i_d', typ: u(undefined, 0) },
            {
                json: 'vesting_infos',
                js: 'vesting_infos',
                typ: u(undefined, a(r('AccountInfoElement'))),
            },
            {
                json: 'account_infos',
                js: 'account_infos',
                typ: u(undefined, a(r('AccountInfoElement'))),
            },
            {
                json: 'vesting_summary',
                js: 'vesting_summary',
                typ: u(undefined, r('VestingSummary')),
            },
        ],
        'any'
    ),
    AccountInfoElement: o(
        [
            { json: 'claimed_amount', js: 'claimed_amount', typ: '' },
            { json: 'end_time', js: 'end_time', typ: 0 },
            { json: 'id', js: 'id', typ: 0 },
            { json: 'locked_amount', js: 'locked_amount', typ: '' },
            { json: 'locker', js: 'locker', typ: '' },
            { json: 'recipient', js: 'recipient', typ: '' },
            { json: 'remaining_amount', js: 'remaining_amount', typ: '' },
            { json: 'start_time', js: 'start_time', typ: 0 },
            { json: 'token', js: 'token', typ: r('AccountInfoToken') },
        ],
        'any'
    ),
    AccountInfoToken: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'hash', js: 'hash', typ: '' },
        ],
        'any'
    ),
    VestingSummary: o(
        [
            { json: 'total_claimed', js: 'total_claimed', typ: '' },
            { json: 'total_locked', js: 'total_locked', typ: '' },
            { json: 'total_remaining', js: 'total_remaining', typ: '' },
        ],
        'any'
    ),
    QueryMsg: o(
        [
            {
                json: 'latest_i_d',
                js: 'latest_i_d',
                typ: u(undefined, m('any')),
            },
            {
                json: 'vesting_infos',
                js: 'vesting_infos',
                typ: u(undefined, r('VestingInfos')),
            },
            {
                json: 'recipients_vesting_infos',
                js: 'recipients_vesting_infos',
                typ: u(undefined, r('RecipientsVestingInfos')),
            },
            {
                json: 'vesting_summary',
                js: 'vesting_summary',
                typ: u(undefined, r('QueryMsgVestingSummary')),
            },
        ],
        'any'
    ),
    RecipientsVestingInfos: o(
        [
            { json: 'page', js: 'page', typ: 0 },
            { json: 'page_size', js: 'page_size', typ: 0 },
            { json: 'recipient', js: 'recipient', typ: '' },
        ],
        'any'
    ),
    VestingInfos: o([{ json: 'ids', js: 'ids', typ: a(0) }], 'any'),
    QueryMsgVestingSummary: o([{ json: 'token', js: 'token', typ: '' }], 'any'),
};
