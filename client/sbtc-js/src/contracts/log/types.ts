// To parse this data:
//
//   import { Convert, HandleAnswer, HandleMsg, QueryAnswer, QueryMsg } from "./file";
//
//   const handleAnswer = Convert.toHandleAnswer(json);
//   const handleMsg = Convert.toHandleMsg(json);
//   const queryAnswer = Convert.toQueryAnswer(json);
//   const queryMsg = Convert.toQueryMsg(json);
//
// These functions will throw an error if the JSON doesn't
// match the expected interface, even if the JSON is valid.

export interface HandleAnswer {
    create_viewing_key: HandleAnswerCreateViewingKey;
}

export interface HandleAnswerCreateViewingKey {
    key: string;
}

export interface HandleMsg {
    setup?: Setup;
    add_events?: AddEvents;
    create_viewing_key?: HandleMsgCreateViewingKey;
    set_viewing_key?: SetViewingKey;
}

export interface AddEvents {
    events: Array<Array<EventObject | string>>;
}

/**
 * tag: 0
 *
 * tag: 1
 *
 * tag: 2
 *
 * tag: 3
 *
 * tag: 4
 *
 * tag: 5
 *
 * tag: 6
 */
export interface EventObject {
    mint_started?: EventMintStarted;
    mint_completed?: EventMintCompleted;
    received_from_treasury?: EventReceivedFromTreasury;
    release_started?: EventReleaseStarted;
    release_request_confirmed?: EventReleaseRequestConfirmed;
    release_completed?: EventReleaseCompleted;
    sent_to_treasury?: EventSentToTreasury;
}

export interface EventMintCompleted {
    address: string;
    amount: number;
    time: number;
    txid: string;
}

export interface EventMintStarted {
    address: string;
    time: number;
}

export interface EventReceivedFromTreasury {
    amount: string;
    time: number;
}

export interface EventReleaseCompleted {
    fee_per_vb: number;
    request_key: number[];
    time: number;
    txid: string;
}

export interface EventReleaseRequestConfirmed {
    block_height: number;
    request_key: number[];
    time: number;
    txid: string;
}

export interface EventReleaseStarted {
    amount: number;
    request_key: number[];
    time: number;
}

export interface EventSentToTreasury {
    amount: string;
    time: number;
}

export interface HandleMsgCreateViewingKey {
    entropy: string;
}

export interface SetViewingKey {
    key: string;
}

export interface Setup {
    config: SetupConfig;
}

export interface SetupConfig {
    gateway: PurpleContractReference;
    treasury: PurpleContractReference;
}

export interface PurpleContractReference {
    address: string;
    hash: string;
}

export interface QueryAnswer {
    config?: QueryAnswerConfig;
    log?: QueryAnswerLog;
}

export interface QueryAnswerConfig {
    gateway: FluffyContractReference;
    treasury: FluffyContractReference;
}

export interface FluffyContractReference {
    address: string;
    hash: string;
}

export interface QueryAnswerLog {
    logs: Event[];
}

/**
 * This enum is used as JSON schema of Query Response.
 *
 * tag: 0
 *
 * tag: 1
 *
 * tag: 2
 *
 * tag: 3
 *
 * tag: 4
 *
 * tag: 5
 *
 * tag: 6
 */
export interface Event {
    mint_started?: EventMintStartedObject;
    mint_completed?: EventMintCompletedObject;
    received_from_treasury?: EventReceivedFromTreasuryObject;
    release_started?: EventReleaseStartedObject;
    release_request_confirmed?: EventReleaseRequestConfirmedObject;
    release_completed?: EventReleaseCompletedObject;
    sent_to_treasury?: EventSentToTreasuryObject;
}

export interface EventMintCompletedObject {
    address: string;
    amount: number;
    time: number;
    txid: string;
}

export interface EventMintStartedObject {
    address: string;
    time: number;
}

export interface EventReceivedFromTreasuryObject {
    amount: string;
    time: number;
}

export interface EventReleaseCompletedObject {
    fee_per_vb: number;
    request_key: number[];
    time: number;
    txid: string;
}

export interface EventReleaseRequestConfirmedObject {
    block_height: number;
    request_key: number[];
    time: number;
    txid: string;
}

export interface EventReleaseStartedObject {
    amount: number;
    request_key: number[];
    time: number;
}

export interface EventSentToTreasuryObject {
    amount: string;
    time: number;
}

export interface QueryMsg {
    log?: QueryMsgLog;
    config?: { [key: string]: any };
}

export interface QueryMsgLog {
    address: string;
    key: string;
    page: number;
    page_size: number;
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
                typ: r('HandleAnswerCreateViewingKey'),
            },
        ],
        'any'
    ),
    HandleAnswerCreateViewingKey: o(
        [{ json: 'key', js: 'key', typ: '' }],
        'any'
    ),
    HandleMsg: o(
        [
            { json: 'setup', js: 'setup', typ: u(undefined, r('Setup')) },
            {
                json: 'add_events',
                js: 'add_events',
                typ: u(undefined, r('AddEvents')),
            },
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
        ],
        'any'
    ),
    AddEvents: o(
        [{ json: 'events', js: 'events', typ: a(a(u(r('EventObject'), ''))) }],
        'any'
    ),
    EventObject: o(
        [
            {
                json: 'mint_started',
                js: 'mint_started',
                typ: u(undefined, r('EventMintStarted')),
            },
            {
                json: 'mint_completed',
                js: 'mint_completed',
                typ: u(undefined, r('EventMintCompleted')),
            },
            {
                json: 'received_from_treasury',
                js: 'received_from_treasury',
                typ: u(undefined, r('EventReceivedFromTreasury')),
            },
            {
                json: 'release_started',
                js: 'release_started',
                typ: u(undefined, r('EventReleaseStarted')),
            },
            {
                json: 'release_request_confirmed',
                js: 'release_request_confirmed',
                typ: u(undefined, r('EventReleaseRequestConfirmed')),
            },
            {
                json: 'release_completed',
                js: 'release_completed',
                typ: u(undefined, r('EventReleaseCompleted')),
            },
            {
                json: 'sent_to_treasury',
                js: 'sent_to_treasury',
                typ: u(undefined, r('EventSentToTreasury')),
            },
        ],
        'any'
    ),
    EventMintCompleted: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'amount', js: 'amount', typ: 0 },
            { json: 'time', js: 'time', typ: 0 },
            { json: 'txid', js: 'txid', typ: '' },
        ],
        'any'
    ),
    EventMintStarted: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'time', js: 'time', typ: 0 },
        ],
        'any'
    ),
    EventReceivedFromTreasury: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'time', js: 'time', typ: 0 },
        ],
        'any'
    ),
    EventReleaseCompleted: o(
        [
            { json: 'fee_per_vb', js: 'fee_per_vb', typ: 0 },
            { json: 'request_key', js: 'request_key', typ: a(0) },
            { json: 'time', js: 'time', typ: 0 },
            { json: 'txid', js: 'txid', typ: '' },
        ],
        'any'
    ),
    EventReleaseRequestConfirmed: o(
        [
            { json: 'block_height', js: 'block_height', typ: 0 },
            { json: 'request_key', js: 'request_key', typ: a(0) },
            { json: 'time', js: 'time', typ: 0 },
            { json: 'txid', js: 'txid', typ: '' },
        ],
        'any'
    ),
    EventReleaseStarted: o(
        [
            { json: 'amount', js: 'amount', typ: 0 },
            { json: 'request_key', js: 'request_key', typ: a(0) },
            { json: 'time', js: 'time', typ: 0 },
        ],
        'any'
    ),
    EventSentToTreasury: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'time', js: 'time', typ: 0 },
        ],
        'any'
    ),
    HandleMsgCreateViewingKey: o(
        [{ json: 'entropy', js: 'entropy', typ: '' }],
        'any'
    ),
    SetViewingKey: o([{ json: 'key', js: 'key', typ: '' }], 'any'),
    Setup: o([{ json: 'config', js: 'config', typ: r('SetupConfig') }], 'any'),
    SetupConfig: o(
        [
            {
                json: 'gateway',
                js: 'gateway',
                typ: r('PurpleContractReference'),
            },
            {
                json: 'treasury',
                js: 'treasury',
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
                json: 'config',
                js: 'config',
                typ: u(undefined, r('QueryAnswerConfig')),
            },
            { json: 'log', js: 'log', typ: u(undefined, r('QueryAnswerLog')) },
        ],
        'any'
    ),
    QueryAnswerConfig: o(
        [
            {
                json: 'gateway',
                js: 'gateway',
                typ: r('FluffyContractReference'),
            },
            {
                json: 'treasury',
                js: 'treasury',
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
    QueryAnswerLog: o(
        [{ json: 'logs', js: 'logs', typ: a(r('Event')) }],
        'any'
    ),
    Event: o(
        [
            {
                json: 'mint_started',
                js: 'mint_started',
                typ: u(undefined, r('EventMintStartedObject')),
            },
            {
                json: 'mint_completed',
                js: 'mint_completed',
                typ: u(undefined, r('EventMintCompletedObject')),
            },
            {
                json: 'received_from_treasury',
                js: 'received_from_treasury',
                typ: u(undefined, r('EventReceivedFromTreasuryObject')),
            },
            {
                json: 'release_started',
                js: 'release_started',
                typ: u(undefined, r('EventReleaseStartedObject')),
            },
            {
                json: 'release_request_confirmed',
                js: 'release_request_confirmed',
                typ: u(undefined, r('EventReleaseRequestConfirmedObject')),
            },
            {
                json: 'release_completed',
                js: 'release_completed',
                typ: u(undefined, r('EventReleaseCompletedObject')),
            },
            {
                json: 'sent_to_treasury',
                js: 'sent_to_treasury',
                typ: u(undefined, r('EventSentToTreasuryObject')),
            },
        ],
        'any'
    ),
    EventMintCompletedObject: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'amount', js: 'amount', typ: 0 },
            { json: 'time', js: 'time', typ: 0 },
            { json: 'txid', js: 'txid', typ: '' },
        ],
        'any'
    ),
    EventMintStartedObject: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'time', js: 'time', typ: 0 },
        ],
        'any'
    ),
    EventReceivedFromTreasuryObject: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'time', js: 'time', typ: 0 },
        ],
        'any'
    ),
    EventReleaseCompletedObject: o(
        [
            { json: 'fee_per_vb', js: 'fee_per_vb', typ: 0 },
            { json: 'request_key', js: 'request_key', typ: a(0) },
            { json: 'time', js: 'time', typ: 0 },
            { json: 'txid', js: 'txid', typ: '' },
        ],
        'any'
    ),
    EventReleaseRequestConfirmedObject: o(
        [
            { json: 'block_height', js: 'block_height', typ: 0 },
            { json: 'request_key', js: 'request_key', typ: a(0) },
            { json: 'time', js: 'time', typ: 0 },
            { json: 'txid', js: 'txid', typ: '' },
        ],
        'any'
    ),
    EventReleaseStartedObject: o(
        [
            { json: 'amount', js: 'amount', typ: 0 },
            { json: 'request_key', js: 'request_key', typ: a(0) },
            { json: 'time', js: 'time', typ: 0 },
        ],
        'any'
    ),
    EventSentToTreasuryObject: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'time', js: 'time', typ: 0 },
        ],
        'any'
    ),
    QueryMsg: o(
        [
            { json: 'log', js: 'log', typ: u(undefined, r('QueryMsgLog')) },
            { json: 'config', js: 'config', typ: u(undefined, m('any')) },
        ],
        'any'
    ),
    QueryMsgLog: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'key', js: 'key', typ: '' },
            { json: 'page', js: 'page', typ: 0 },
            { json: 'page_size', js: 'page_size', typ: 0 },
        ],
        'any'
    ),
};
