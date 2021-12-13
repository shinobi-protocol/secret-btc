// To parse this data:
//
//   import { Convert, CustomQueryAnswer, HandleMsgForCustomHandleMsg, InitMsg, QueryMsgForCustomQueryMsg } from "./file";
//
//   const arrayOfOperation = Convert.toArrayOfOperation(json);
//   const customQueryAnswer = Convert.toCustomQueryAnswer(json);
//   const handleMsgForCustomHandleMsg = Convert.toHandleMsgForCustomHandleMsg(json);
//   const initMsg = Convert.toInitMsg(json);
//   const queryMsgForCustomQueryMsg = Convert.toQueryMsgForCustomQueryMsg(json);
//
// These functions will throw an error if the JSON doesn't
// match the expected interface, even if the JSON is valid.

export interface ArrayOfOperation {
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

export interface CustomQueryAnswer {
    config?: CustomQueryAnswerConfig;
    total_minted_sbtc?: string;
}

export interface CustomQueryAnswerConfig {
    developer_address: string;
    gateway: PurpleContractReference;
    owner: string;
    shuriken: PurpleContractReference;
    snb: PurpleContractReference;
    treasury: PurpleContractReference;
}

export interface PurpleContractReference {
    address: string;
    hash: string;
}

export interface HandleMsgForCustomHandleMsg {
    migrate?: Migrate;
    send_mint_reward?: SendMintReward;
    receive_release_fee?: ReceiveReleaseFee;
    mint_bitcoin_s_p_v_reward?: MintBitcoinSPVReward;
    mint_s_f_p_s_reward?: MintSFPSReward;
    custom?: HandleMsgForCustomHandleMsgCustom;
}

export interface HandleMsgForCustomHandleMsgCustom {
    custom_msg: CustomHandleMsg;
}

export interface CustomHandleMsg {
    transfer_ownership: TransferOwnership;
}

export interface TransferOwnership {
    owner: string;
}

export interface Migrate {
    new_finance_admin: NewFinanceAdminObject;
}

export interface NewFinanceAdminObject {
    address: string;
    hash: string;
}

export interface MintBitcoinSPVReward {
    best_height: number;
    executer: string;
}

export interface MintSFPSReward {
    best_height: number;
    executer: string;
}

export interface ReceiveReleaseFee {
    releaser: string;
    sbtc_release_amount: string;
    sbtc_total_supply: string;
}

export interface SendMintReward {
    minter: string;
    sbtc_mint_amount: string;
    sbtc_total_supply: string;
}

export interface InitMsg {
    config: InitMsgConfig;
}

export interface InitMsgConfig {
    developer_address: string;
    gateway: FluffyContractReference;
    owner: string;
    shuriken: FluffyContractReference;
    snb: FluffyContractReference;
    treasury: FluffyContractReference;
}

export interface FluffyContractReference {
    address: string;
    hash: string;
}

export interface QueryMsgForCustomQueryMsg {
    mint_reward?: MintReward;
    release_fee?: ReleaseFee;
    custom?: QueryMsgForCustomQueryMsgCustom;
}

export interface QueryMsgForCustomQueryMsgCustom {
    custom_msg: CustomQueryMsg;
}

export interface CustomQueryMsg {
    config?: { [key: string]: any };
    total_minted_sbtc?: { [key: string]: any };
}

export interface MintReward {
    minter: string;
    sbtc_mint_amount: string;
    sbtc_total_supply: string;
}

export interface ReleaseFee {
    releaser: string;
    sbtc_release_amount: string;
    sbtc_total_supply: string;
}

// Converts JSON strings to/from your types
// and asserts the results of JSON.parse at runtime
export class Convert {
    public static toArrayOfOperation(json: string): ArrayOfOperation[] {
        return cast(JSON.parse(json), a(r('ArrayOfOperation')));
    }

    public static arrayOfOperationToJson(value: ArrayOfOperation[]): string {
        return JSON.stringify(uncast(value, a(r('ArrayOfOperation'))), null, 2);
    }

    public static toCustomQueryAnswer(json: string): CustomQueryAnswer {
        return cast(JSON.parse(json), r('CustomQueryAnswer'));
    }

    public static customQueryAnswerToJson(value: CustomQueryAnswer): string {
        return JSON.stringify(uncast(value, r('CustomQueryAnswer')), null, 2);
    }

    public static toHandleMsgForCustomHandleMsg(
        json: string
    ): HandleMsgForCustomHandleMsg {
        return cast(JSON.parse(json), r('HandleMsgForCustomHandleMsg'));
    }

    public static handleMsgForCustomHandleMsgToJson(
        value: HandleMsgForCustomHandleMsg
    ): string {
        return JSON.stringify(
            uncast(value, r('HandleMsgForCustomHandleMsg')),
            null,
            2
        );
    }

    public static toInitMsg(json: string): InitMsg {
        return cast(JSON.parse(json), r('InitMsg'));
    }

    public static initMsgToJson(value: InitMsg): string {
        return JSON.stringify(uncast(value, r('InitMsg')), null, 2);
    }

    public static toQueryMsgForCustomQueryMsg(
        json: string
    ): QueryMsgForCustomQueryMsg {
        return cast(JSON.parse(json), r('QueryMsgForCustomQueryMsg'));
    }

    public static queryMsgForCustomQueryMsgToJson(
        value: QueryMsgForCustomQueryMsg
    ): string {
        return JSON.stringify(
            uncast(value, r('QueryMsgForCustomQueryMsg')),
            null,
            2
        );
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
    ArrayOfOperation: o(
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
    CustomQueryAnswer: o(
        [
            {
                json: 'config',
                js: 'config',
                typ: u(undefined, r('CustomQueryAnswerConfig')),
            },
            {
                json: 'total_minted_sbtc',
                js: 'total_minted_sbtc',
                typ: u(undefined, ''),
            },
        ],
        'any'
    ),
    CustomQueryAnswerConfig: o(
        [
            { json: 'developer_address', js: 'developer_address', typ: '' },
            {
                json: 'gateway',
                js: 'gateway',
                typ: r('PurpleContractReference'),
            },
            { json: 'owner', js: 'owner', typ: '' },
            {
                json: 'shuriken',
                js: 'shuriken',
                typ: r('PurpleContractReference'),
            },
            { json: 'snb', js: 'snb', typ: r('PurpleContractReference') },
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
    HandleMsgForCustomHandleMsg: o(
        [
            { json: 'migrate', js: 'migrate', typ: u(undefined, r('Migrate')) },
            {
                json: 'send_mint_reward',
                js: 'send_mint_reward',
                typ: u(undefined, r('SendMintReward')),
            },
            {
                json: 'receive_release_fee',
                js: 'receive_release_fee',
                typ: u(undefined, r('ReceiveReleaseFee')),
            },
            {
                json: 'mint_bitcoin_s_p_v_reward',
                js: 'mint_bitcoin_s_p_v_reward',
                typ: u(undefined, r('MintBitcoinSPVReward')),
            },
            {
                json: 'mint_s_f_p_s_reward',
                js: 'mint_s_f_p_s_reward',
                typ: u(undefined, r('MintSFPSReward')),
            },
            {
                json: 'custom',
                js: 'custom',
                typ: u(undefined, r('HandleMsgForCustomHandleMsgCustom')),
            },
        ],
        'any'
    ),
    HandleMsgForCustomHandleMsgCustom: o(
        [{ json: 'custom_msg', js: 'custom_msg', typ: r('CustomHandleMsg') }],
        'any'
    ),
    CustomHandleMsg: o(
        [
            {
                json: 'transfer_ownership',
                js: 'transfer_ownership',
                typ: r('TransferOwnership'),
            },
        ],
        'any'
    ),
    TransferOwnership: o([{ json: 'owner', js: 'owner', typ: '' }], 'any'),
    Migrate: o(
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
    MintBitcoinSPVReward: o(
        [
            { json: 'best_height', js: 'best_height', typ: 0 },
            { json: 'executer', js: 'executer', typ: '' },
        ],
        'any'
    ),
    MintSFPSReward: o(
        [
            { json: 'best_height', js: 'best_height', typ: 0 },
            { json: 'executer', js: 'executer', typ: '' },
        ],
        'any'
    ),
    ReceiveReleaseFee: o(
        [
            { json: 'releaser', js: 'releaser', typ: '' },
            { json: 'sbtc_release_amount', js: 'sbtc_release_amount', typ: '' },
            { json: 'sbtc_total_supply', js: 'sbtc_total_supply', typ: '' },
        ],
        'any'
    ),
    SendMintReward: o(
        [
            { json: 'minter', js: 'minter', typ: '' },
            { json: 'sbtc_mint_amount', js: 'sbtc_mint_amount', typ: '' },
            { json: 'sbtc_total_supply', js: 'sbtc_total_supply', typ: '' },
        ],
        'any'
    ),
    InitMsg: o(
        [{ json: 'config', js: 'config', typ: r('InitMsgConfig') }],
        'any'
    ),
    InitMsgConfig: o(
        [
            { json: 'developer_address', js: 'developer_address', typ: '' },
            {
                json: 'gateway',
                js: 'gateway',
                typ: r('FluffyContractReference'),
            },
            { json: 'owner', js: 'owner', typ: '' },
            {
                json: 'shuriken',
                js: 'shuriken',
                typ: r('FluffyContractReference'),
            },
            { json: 'snb', js: 'snb', typ: r('FluffyContractReference') },
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
    QueryMsgForCustomQueryMsg: o(
        [
            {
                json: 'mint_reward',
                js: 'mint_reward',
                typ: u(undefined, r('MintReward')),
            },
            {
                json: 'release_fee',
                js: 'release_fee',
                typ: u(undefined, r('ReleaseFee')),
            },
            {
                json: 'custom',
                js: 'custom',
                typ: u(undefined, r('QueryMsgForCustomQueryMsgCustom')),
            },
        ],
        'any'
    ),
    QueryMsgForCustomQueryMsgCustom: o(
        [{ json: 'custom_msg', js: 'custom_msg', typ: r('CustomQueryMsg') }],
        'any'
    ),
    CustomQueryMsg: o(
        [
            { json: 'config', js: 'config', typ: u(undefined, m('any')) },
            {
                json: 'total_minted_sbtc',
                js: 'total_minted_sbtc',
                typ: u(undefined, m('any')),
            },
        ],
        'any'
    ),
    MintReward: o(
        [
            { json: 'minter', js: 'minter', typ: '' },
            { json: 'sbtc_mint_amount', js: 'sbtc_mint_amount', typ: '' },
            { json: 'sbtc_total_supply', js: 'sbtc_total_supply', typ: '' },
        ],
        'any'
    ),
    ReleaseFee: o(
        [
            { json: 'releaser', js: 'releaser', typ: '' },
            { json: 'sbtc_release_amount', js: 'sbtc_release_amount', typ: '' },
            { json: 'sbtc_total_supply', js: 'sbtc_total_supply', typ: '' },
        ],
        'any'
    ),
};
