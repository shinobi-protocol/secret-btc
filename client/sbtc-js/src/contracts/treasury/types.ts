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
    operate?: Operate;
    transfer_ownership?: TransferOwnership;
}

export interface Operate {
    operations: Operation[];
}

export interface Operation {
    send?: Send;
    receive_from?: ReceiveFrom;
}

export interface ReceiveFrom {
    amount: string;
    from: string;
}

export interface Send {
    amount: string;
    to: string;
}

export interface TransferOwnership {
    owner: string;
}

export interface InitMsg {
    config: InitMsgConfig;
}

export interface InitMsgConfig {
    log: PurpleContractReference;
    owner: string;
    snb: PurpleContractReference;
}

export interface PurpleContractReference {
    address: string;
    hash: string;
}

export interface QueryAnswer {
    config: QueryAnswerConfig;
}

export interface QueryAnswerConfig {
    log: FluffyContractReference;
    owner: string;
    snb: FluffyContractReference;
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
            { json: 'operate', js: 'operate', typ: u(undefined, r('Operate')) },
            {
                json: 'transfer_ownership',
                js: 'transfer_ownership',
                typ: u(undefined, r('TransferOwnership')),
            },
        ],
        'any'
    ),
    Operate: o(
        [{ json: 'operations', js: 'operations', typ: a(r('Operation')) }],
        'any'
    ),
    Operation: o(
        [
            { json: 'send', js: 'send', typ: u(undefined, r('Send')) },
            {
                json: 'receive_from',
                js: 'receive_from',
                typ: u(undefined, r('ReceiveFrom')),
            },
        ],
        'any'
    ),
    ReceiveFrom: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'from', js: 'from', typ: '' },
        ],
        'any'
    ),
    Send: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'to', js: 'to', typ: '' },
        ],
        'any'
    ),
    TransferOwnership: o([{ json: 'owner', js: 'owner', typ: '' }], 'any'),
    InitMsg: o(
        [{ json: 'config', js: 'config', typ: r('InitMsgConfig') }],
        'any'
    ),
    InitMsgConfig: o(
        [
            { json: 'log', js: 'log', typ: r('PurpleContractReference') },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'snb', js: 'snb', typ: r('PurpleContractReference') },
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
            { json: 'log', js: 'log', typ: r('FluffyContractReference') },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'snb', js: 'snb', typ: r('FluffyContractReference') },
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
};
