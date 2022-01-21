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
    submit_transaction: HandleAnswerSubmitTransaction;
}

export interface HandleAnswerSubmitTransaction {
    transaction_id: number;
}

export interface HandleMsg {
    change_config?: ChangeConfig;
    submit_transaction?: HandleMsgSubmitTransaction;
    sign_transaction?: SignTransaction;
}

export interface ChangeConfig {
    config: ChangeConfigConfig;
}

export interface ChangeConfigConfig {
    required: number;
    signers: string[];
}

export interface SignTransaction {
    transaction_id: number;
}

export interface HandleMsgSubmitTransaction {
    transaction: SubmitTransactionTransaction;
}

export interface SubmitTransactionTransaction {
    callback_code_hash: string;
    contract_addr: string;
    msg: string;
    send: PurpleCoin[];
}

export interface PurpleCoin {
    amount: string;
    denom: string;
}

export interface InitMsg {
    config: InitMsgConfig;
}

export interface InitMsgConfig {
    required: number;
    signers: string[];
}

export interface QueryAnswer {
    transaction_status?: TransactionStatus;
    multisig_status?: MultisigStatus;
}

export interface MultisigStatus {
    config: MultisigStatusConfig;
    transaction_count: number;
}

export interface MultisigStatusConfig {
    required: number;
    signers: string[];
}

export interface TransactionStatus {
    config: MultisigStatusConfig;
    signed_by: number[];
    transaction: TransactionStatusTransaction;
}

export interface TransactionStatusTransaction {
    callback_code_hash: string;
    contract_addr: string;
    msg: string;
    send: FluffyCoin[];
}

export interface FluffyCoin {
    amount: string;
    denom: string;
}

export interface QueryMsg {
    transaction_status?: QueryMsgTransactionStatus;
    multisig_status?: { [key: string]: any };
}

export interface QueryMsgTransactionStatus {
    transaction_id: number;
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
                json: 'submit_transaction',
                js: 'submit_transaction',
                typ: r('HandleAnswerSubmitTransaction'),
            },
        ],
        'any'
    ),
    HandleAnswerSubmitTransaction: o(
        [{ json: 'transaction_id', js: 'transaction_id', typ: 0 }],
        'any'
    ),
    HandleMsg: o(
        [
            {
                json: 'change_config',
                js: 'change_config',
                typ: u(undefined, r('ChangeConfig')),
            },
            {
                json: 'submit_transaction',
                js: 'submit_transaction',
                typ: u(undefined, r('HandleMsgSubmitTransaction')),
            },
            {
                json: 'sign_transaction',
                js: 'sign_transaction',
                typ: u(undefined, r('SignTransaction')),
            },
        ],
        'any'
    ),
    ChangeConfig: o(
        [{ json: 'config', js: 'config', typ: r('ChangeConfigConfig') }],
        'any'
    ),
    ChangeConfigConfig: o(
        [
            { json: 'required', js: 'required', typ: 0 },
            { json: 'signers', js: 'signers', typ: a('') },
        ],
        'any'
    ),
    SignTransaction: o(
        [{ json: 'transaction_id', js: 'transaction_id', typ: 0 }],
        'any'
    ),
    HandleMsgSubmitTransaction: o(
        [
            {
                json: 'transaction',
                js: 'transaction',
                typ: r('SubmitTransactionTransaction'),
            },
        ],
        'any'
    ),
    SubmitTransactionTransaction: o(
        [
            { json: 'callback_code_hash', js: 'callback_code_hash', typ: '' },
            { json: 'contract_addr', js: 'contract_addr', typ: '' },
            { json: 'msg', js: 'msg', typ: '' },
            { json: 'send', js: 'send', typ: a(r('PurpleCoin')) },
        ],
        'any'
    ),
    PurpleCoin: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'denom', js: 'denom', typ: '' },
        ],
        'any'
    ),
    InitMsg: o(
        [{ json: 'config', js: 'config', typ: r('InitMsgConfig') }],
        'any'
    ),
    InitMsgConfig: o(
        [
            { json: 'required', js: 'required', typ: 0 },
            { json: 'signers', js: 'signers', typ: a('') },
        ],
        'any'
    ),
    QueryAnswer: o(
        [
            {
                json: 'transaction_status',
                js: 'transaction_status',
                typ: u(undefined, r('TransactionStatus')),
            },
            {
                json: 'multisig_status',
                js: 'multisig_status',
                typ: u(undefined, r('MultisigStatus')),
            },
        ],
        'any'
    ),
    MultisigStatus: o(
        [
            { json: 'config', js: 'config', typ: r('MultisigStatusConfig') },
            { json: 'transaction_count', js: 'transaction_count', typ: 0 },
        ],
        'any'
    ),
    MultisigStatusConfig: o(
        [
            { json: 'required', js: 'required', typ: 0 },
            { json: 'signers', js: 'signers', typ: a('') },
        ],
        'any'
    ),
    TransactionStatus: o(
        [
            { json: 'config', js: 'config', typ: r('MultisigStatusConfig') },
            { json: 'signed_by', js: 'signed_by', typ: a(0) },
            {
                json: 'transaction',
                js: 'transaction',
                typ: r('TransactionStatusTransaction'),
            },
        ],
        'any'
    ),
    TransactionStatusTransaction: o(
        [
            { json: 'callback_code_hash', js: 'callback_code_hash', typ: '' },
            { json: 'contract_addr', js: 'contract_addr', typ: '' },
            { json: 'msg', js: 'msg', typ: '' },
            { json: 'send', js: 'send', typ: a(r('FluffyCoin')) },
        ],
        'any'
    ),
    FluffyCoin: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'denom', js: 'denom', typ: '' },
        ],
        'any'
    ),
    QueryMsg: o(
        [
            {
                json: 'transaction_status',
                js: 'transaction_status',
                typ: u(undefined, r('QueryMsgTransactionStatus')),
            },
            {
                json: 'multisig_status',
                js: 'multisig_status',
                typ: u(undefined, m('any')),
            },
        ],
        'any'
    ),
    QueryMsgTransactionStatus: o(
        [{ json: 'transaction_id', js: 'transaction_id', typ: 0 }],
        'any'
    ),
};
