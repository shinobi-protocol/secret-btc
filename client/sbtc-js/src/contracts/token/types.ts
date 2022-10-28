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
    deposit?: HandleAnswerDeposit;
    redeem?: HandleAnswerRedeem;
    transfer?: HandleAnswerTransfer;
    send?: HandleAnswerSend;
    batch_transfer?: HandleAnswerBatchTransfer;
    batch_send?: HandleAnswerBatchSend;
    burn?: HandleAnswerBurn;
    register_receive?: HandleAnswerRegisterReceive;
    create_viewing_key?: HandleAnswerCreateViewingKey;
    set_viewing_key?: HandleAnswerSetViewingKey;
    increase_allowance?: HandleAnswerIncreaseAllowance;
    decrease_allowance?: HandleAnswerDecreaseAllowance;
    transfer_from?: HandleAnswerTransferFrom;
    send_from?: HandleAnswerSendFrom;
    batch_transfer_from?: HandleAnswerBatchTransferFrom;
    batch_send_from?: HandleAnswerBatchSendFrom;
    burn_from?: HandleAnswerBurnFrom;
    batch_burn_from?: HandleAnswerBatchBurnFrom;
    mint?: HandleAnswerMint;
    batch_mint?: HandleAnswerBatchMint;
    add_minters?: HandleAnswerAddMinters;
    remove_minters?: HandleAnswerRemoveMinters;
    set_minters?: HandleAnswerSetMinters;
    change_admin?: HandleAnswerChangeAdmin;
    set_contract_status?: HandleAnswerSetContractStatus;
    revoke_permit?: HandleAnswerRevokePermit;
}

export interface HandleAnswerAddMinters {
    status: ResponseStatus;
}

export enum ResponseStatus {
    Failure = 'failure',
    Success = 'success',
}

export interface HandleAnswerBatchBurnFrom {
    status: ResponseStatus;
}

export interface HandleAnswerBatchMint {
    status: ResponseStatus;
}

export interface HandleAnswerBatchSend {
    status: ResponseStatus;
}

export interface HandleAnswerBatchSendFrom {
    status: ResponseStatus;
}

export interface HandleAnswerBatchTransfer {
    status: ResponseStatus;
}

export interface HandleAnswerBatchTransferFrom {
    status: ResponseStatus;
}

export interface HandleAnswerBurn {
    status: ResponseStatus;
}

export interface HandleAnswerBurnFrom {
    status: ResponseStatus;
}

export interface HandleAnswerChangeAdmin {
    status: ResponseStatus;
}

export interface HandleAnswerCreateViewingKey {
    key: string;
}

export interface HandleAnswerDecreaseAllowance {
    allowance: string;
    owner: string;
    spender: string;
}

export interface HandleAnswerDeposit {
    status: ResponseStatus;
}

export interface HandleAnswerIncreaseAllowance {
    allowance: string;
    owner: string;
    spender: string;
}

export interface HandleAnswerMint {
    status: ResponseStatus;
}

export interface HandleAnswerRedeem {
    status: ResponseStatus;
}

export interface HandleAnswerRegisterReceive {
    status: ResponseStatus;
}

export interface HandleAnswerRemoveMinters {
    status: ResponseStatus;
}

export interface HandleAnswerRevokePermit {
    status: ResponseStatus;
}

export interface HandleAnswerSend {
    status: ResponseStatus;
}

export interface HandleAnswerSendFrom {
    status: ResponseStatus;
}

export interface HandleAnswerSetContractStatus {
    status: ResponseStatus;
}

export interface HandleAnswerSetMinters {
    status: ResponseStatus;
}

export interface HandleAnswerSetViewingKey {
    status: ResponseStatus;
}

export interface HandleAnswerTransfer {
    status: ResponseStatus;
}

export interface HandleAnswerTransferFrom {
    status: ResponseStatus;
}

export interface HandleMsg {
    redeem?: HandleMsgRedeem;
    deposit?: HandleMsgDeposit;
    transfer?: HandleMsgTransfer;
    send?: HandleMsgSend;
    batch_transfer?: HandleMsgBatchTransfer;
    batch_send?: HandleMsgBatchSend;
    burn?: HandleMsgBurn;
    register_receive?: HandleMsgRegisterReceive;
    create_viewing_key?: HandleMsgCreateViewingKey;
    set_viewing_key?: HandleMsgSetViewingKey;
    increase_allowance?: HandleMsgIncreaseAllowance;
    decrease_allowance?: HandleMsgDecreaseAllowance;
    transfer_from?: HandleMsgTransferFrom;
    send_from?: HandleMsgSendFrom;
    batch_transfer_from?: HandleMsgBatchTransferFrom;
    batch_send_from?: HandleMsgBatchSendFrom;
    burn_from?: HandleMsgBurnFrom;
    batch_burn_from?: HandleMsgBatchBurnFrom;
    mint?: HandleMsgMint;
    batch_mint?: HandleMsgBatchMint;
    add_minters?: HandleMsgAddMinters;
    remove_minters?: HandleMsgRemoveMinters;
    set_minters?: HandleMsgSetMinters;
    change_admin?: HandleMsgChangeAdmin;
    set_contract_status?: HandleMsgSetContractStatus;
    revoke_permit?: HandleMsgRevokePermit;
}

export interface HandleMsgAddMinters {
    minters: string[];
    padding?: null | string;
}

export interface HandleMsgBatchBurnFrom {
    actions: BurnFromAction[];
    padding?: null | string;
}

export interface BurnFromAction {
    amount: string;
    memo?: null | string;
    owner: string;
}

export interface HandleMsgBatchMint {
    actions: MintAction[];
    padding?: null | string;
}

export interface MintAction {
    amount: string;
    memo?: null | string;
    recipient: string;
}

export interface HandleMsgBatchSend {
    actions: SendAction[];
    padding?: null | string;
}

export interface SendAction {
    amount: string;
    memo?: null | string;
    msg?: null | string;
    recipient: string;
    recipient_code_hash?: null | string;
}

export interface HandleMsgBatchSendFrom {
    actions: SendFromAction[];
    padding?: null | string;
}

export interface SendFromAction {
    amount: string;
    memo?: null | string;
    msg?: null | string;
    owner: string;
    recipient: string;
    recipient_code_hash?: null | string;
}

export interface HandleMsgBatchTransfer {
    actions: TransferAction[];
    padding?: null | string;
}

export interface TransferAction {
    amount: string;
    memo?: null | string;
    recipient: string;
}

export interface HandleMsgBatchTransferFrom {
    actions: TransferFromAction[];
    padding?: null | string;
}

export interface TransferFromAction {
    amount: string;
    memo?: null | string;
    owner: string;
    recipient: string;
}

export interface HandleMsgBurn {
    amount: string;
    memo?: null | string;
    padding?: null | string;
}

export interface HandleMsgBurnFrom {
    amount: string;
    memo?: null | string;
    owner: string;
    padding?: null | string;
}

export interface HandleMsgChangeAdmin {
    address: string;
    padding?: null | string;
}

export interface HandleMsgCreateViewingKey {
    entropy: string;
    padding?: null | string;
}

export interface HandleMsgDecreaseAllowance {
    amount: string;
    expiration?: number | null;
    padding?: null | string;
    spender: string;
}

export interface HandleMsgDeposit {
    padding?: null | string;
}

export interface HandleMsgIncreaseAllowance {
    amount: string;
    expiration?: number | null;
    padding?: null | string;
    spender: string;
}

export interface HandleMsgMint {
    amount: string;
    memo?: null | string;
    padding?: null | string;
    recipient: string;
}

export interface HandleMsgRedeem {
    amount: string;
    denom?: null | string;
    padding?: null | string;
}

export interface HandleMsgRegisterReceive {
    code_hash: string;
    padding?: null | string;
}

export interface HandleMsgRemoveMinters {
    minters: string[];
    padding?: null | string;
}

export interface HandleMsgRevokePermit {
    padding?: null | string;
    permit_name: string;
}

export interface HandleMsgSend {
    amount: string;
    memo?: null | string;
    msg?: null | string;
    padding?: null | string;
    recipient: string;
    recipient_code_hash?: null | string;
}

export interface HandleMsgSendFrom {
    amount: string;
    memo?: null | string;
    msg?: null | string;
    owner: string;
    padding?: null | string;
    recipient: string;
    recipient_code_hash?: null | string;
}

export interface HandleMsgSetContractStatus {
    level: ContractStatusLevel;
    padding?: null | string;
}

export enum ContractStatusLevel {
    NormalRun = 'normal_run',
    StopAll = 'stop_all',
    StopAllButRedeems = 'stop_all_but_redeems',
}

export interface HandleMsgSetMinters {
    minters: string[];
    padding?: null | string;
}

export interface HandleMsgSetViewingKey {
    key: string;
    padding?: null | string;
}

export interface HandleMsgTransfer {
    amount: string;
    memo?: null | string;
    padding?: null | string;
    recipient: string;
}

export interface HandleMsgTransferFrom {
    amount: string;
    memo?: null | string;
    owner: string;
    padding?: null | string;
    recipient: string;
}

export interface InitMsg {
    admin?: null | string;
    config?: null | InitConfig;
    decimals: number;
    initial_balances?: InitialBalance[] | null;
    name: string;
    prng_seed: string;
    symbol: string;
}

/**
 * This type represents optional configuration values which can be overridden. All values
 * are optional and have defaults which are more private by default, but can be overridden
 * if necessary
 */
export interface InitConfig {
    /**
     * Indicates whether burn functionality should be enabled default: False
     */
    enable_burn?: boolean | null;
    /**
     * Indicates whether deposit functionality should be enabled default: False
     */
    enable_deposit?: boolean | null;
    /**
     * Indicates whether mint functionality should be enabled default: False
     */
    enable_mint?: boolean | null;
    /**
     * Indicates whether redeem functionality should be enabled default: False
     */
    enable_redeem?: boolean | null;
    /**
     * Indicates whether the total supply is public or should be kept secret. default: False
     */
    public_total_supply?: boolean | null;
}

export interface InitialBalance {
    address: string;
    amount: string;
}

export interface QueryAnswer {
    token_info?: TokenInfo;
    token_config?: TokenConfig;
    contract_status?: ContractStatus;
    exchange_rate?: ExchangeRate;
    allowance?: QueryAnswerAllowance;
    balance?: QueryAnswerBalance;
    transfer_history?: QueryAnswerTransferHistory;
    transaction_history?: QueryAnswerTransactionHistory;
    viewing_key_error?: ViewingKeyError;
    minters?: Minters;
}

export interface QueryAnswerAllowance {
    allowance: string;
    expiration?: number | null;
    owner: string;
    spender: string;
}

export interface QueryAnswerBalance {
    amount: string;
}

export interface ContractStatus {
    status: ContractStatusLevel;
}

export interface ExchangeRate {
    denom: string;
    rate: string;
}

export interface Minters {
    minters: string[];
}

export interface TokenConfig {
    burn_enabled: boolean;
    deposit_enabled: boolean;
    mint_enabled: boolean;
    public_total_supply: boolean;
    redeem_enabled: boolean;
}

export interface TokenInfo {
    decimals: number;
    name: string;
    symbol: string;
    total_supply?: null | string;
}

export interface QueryAnswerTransactionHistory {
    total?: number | null;
    txs: RichTx[];
}

export interface RichTx {
    action: TxAction;
    block_height: number;
    block_time: number;
    coins: Coin;
    id: number;
    memo?: null | string;
}

export interface TxAction {
    transfer?: TxActionTransfer;
    mint?: TxActionMint;
    burn?: TxActionBurn;
    deposit?: { [key: string]: any };
    redeem?: { [key: string]: any };
}

export interface TxActionBurn {
    burner: string;
    owner: string;
}

export interface TxActionMint {
    minter: string;
    recipient: string;
}

export interface TxActionTransfer {
    from: string;
    recipient: string;
    sender: string;
}

export interface Coin {
    amount: string;
    denom: string;
}

export interface QueryAnswerTransferHistory {
    total?: number | null;
    txs: Tx[];
}

export interface Tx {
    block_height?: number | null;
    block_time?: number | null;
    coins: Coin;
    from: string;
    id: number;
    memo?: null | string;
    receiver: string;
    sender: string;
}

export interface ViewingKeyError {
    msg: string;
}

export interface QueryMsg {
    token_info?: { [key: string]: any };
    token_config?: { [key: string]: any };
    contract_status?: { [key: string]: any };
    exchange_rate?: { [key: string]: any };
    allowance?: QueryMsgAllowance;
    balance?: QueryMsgBalance;
    transfer_history?: QueryMsgTransferHistory;
    transaction_history?: QueryMsgTransactionHistory;
    minters?: { [key: string]: any };
    with_permit?: WithPermit;
}

export interface QueryMsgAllowance {
    key: string;
    owner: string;
    spender: string;
}

export interface QueryMsgBalance {
    address: string;
    key: string;
}

export interface QueryMsgTransactionHistory {
    address: string;
    key: string;
    page?: number | null;
    page_size: number;
}

export interface QueryMsgTransferHistory {
    address: string;
    key: string;
    page?: number | null;
    page_size: number;
}

export interface WithPermit {
    permit: PermitForTokenPermissions;
    query: QueryWithPermit;
}

export interface PermitForTokenPermissions {
    params: PermitParamsForTokenPermissions;
    signature: PermitSignature;
}

export interface PermitParamsForTokenPermissions {
    allowed_tokens: string[];
    chain_id: string;
    permissions: TokenPermissions[];
    permit_name: string;
}

export enum TokenPermissions {
    Allowance = 'allowance',
    Balance = 'balance',
    History = 'history',
    Owner = 'owner',
}

export interface PermitSignature {
    pub_key: PubKey;
    signature: string;
}

export interface PubKey {
    /**
     * ignored, but must be "tendermint/PubKeySecp256k1" otherwise the verification will fail
     */
    type: string;
    /**
     * Secp256k1 PubKey
     */
    value: string;
}

export interface QueryWithPermit {
    allowance?: QueryWithPermitAllowance;
    balance?: { [key: string]: any };
    transfer_history?: QueryWithPermitTransferHistory;
    transaction_history?: QueryWithPermitTransactionHistory;
}

export interface QueryWithPermitAllowance {
    owner: string;
    spender: string;
}

export interface QueryWithPermitTransactionHistory {
    page?: number | null;
    page_size: number;
}

export interface QueryWithPermitTransferHistory {
    page?: number | null;
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
                json: 'deposit',
                js: 'deposit',
                typ: u(undefined, r('HandleAnswerDeposit')),
            },
            {
                json: 'redeem',
                js: 'redeem',
                typ: u(undefined, r('HandleAnswerRedeem')),
            },
            {
                json: 'transfer',
                js: 'transfer',
                typ: u(undefined, r('HandleAnswerTransfer')),
            },
            {
                json: 'send',
                js: 'send',
                typ: u(undefined, r('HandleAnswerSend')),
            },
            {
                json: 'batch_transfer',
                js: 'batch_transfer',
                typ: u(undefined, r('HandleAnswerBatchTransfer')),
            },
            {
                json: 'batch_send',
                js: 'batch_send',
                typ: u(undefined, r('HandleAnswerBatchSend')),
            },
            {
                json: 'burn',
                js: 'burn',
                typ: u(undefined, r('HandleAnswerBurn')),
            },
            {
                json: 'register_receive',
                js: 'register_receive',
                typ: u(undefined, r('HandleAnswerRegisterReceive')),
            },
            {
                json: 'create_viewing_key',
                js: 'create_viewing_key',
                typ: u(undefined, r('HandleAnswerCreateViewingKey')),
            },
            {
                json: 'set_viewing_key',
                js: 'set_viewing_key',
                typ: u(undefined, r('HandleAnswerSetViewingKey')),
            },
            {
                json: 'increase_allowance',
                js: 'increase_allowance',
                typ: u(undefined, r('HandleAnswerIncreaseAllowance')),
            },
            {
                json: 'decrease_allowance',
                js: 'decrease_allowance',
                typ: u(undefined, r('HandleAnswerDecreaseAllowance')),
            },
            {
                json: 'transfer_from',
                js: 'transfer_from',
                typ: u(undefined, r('HandleAnswerTransferFrom')),
            },
            {
                json: 'send_from',
                js: 'send_from',
                typ: u(undefined, r('HandleAnswerSendFrom')),
            },
            {
                json: 'batch_transfer_from',
                js: 'batch_transfer_from',
                typ: u(undefined, r('HandleAnswerBatchTransferFrom')),
            },
            {
                json: 'batch_send_from',
                js: 'batch_send_from',
                typ: u(undefined, r('HandleAnswerBatchSendFrom')),
            },
            {
                json: 'burn_from',
                js: 'burn_from',
                typ: u(undefined, r('HandleAnswerBurnFrom')),
            },
            {
                json: 'batch_burn_from',
                js: 'batch_burn_from',
                typ: u(undefined, r('HandleAnswerBatchBurnFrom')),
            },
            {
                json: 'mint',
                js: 'mint',
                typ: u(undefined, r('HandleAnswerMint')),
            },
            {
                json: 'batch_mint',
                js: 'batch_mint',
                typ: u(undefined, r('HandleAnswerBatchMint')),
            },
            {
                json: 'add_minters',
                js: 'add_minters',
                typ: u(undefined, r('HandleAnswerAddMinters')),
            },
            {
                json: 'remove_minters',
                js: 'remove_minters',
                typ: u(undefined, r('HandleAnswerRemoveMinters')),
            },
            {
                json: 'set_minters',
                js: 'set_minters',
                typ: u(undefined, r('HandleAnswerSetMinters')),
            },
            {
                json: 'change_admin',
                js: 'change_admin',
                typ: u(undefined, r('HandleAnswerChangeAdmin')),
            },
            {
                json: 'set_contract_status',
                js: 'set_contract_status',
                typ: u(undefined, r('HandleAnswerSetContractStatus')),
            },
            {
                json: 'revoke_permit',
                js: 'revoke_permit',
                typ: u(undefined, r('HandleAnswerRevokePermit')),
            },
        ],
        'any'
    ),
    HandleAnswerAddMinters: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerBatchBurnFrom: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerBatchMint: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerBatchSend: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerBatchSendFrom: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerBatchTransfer: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerBatchTransferFrom: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerBurn: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerBurnFrom: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerChangeAdmin: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerCreateViewingKey: o(
        [{ json: 'key', js: 'key', typ: '' }],
        'any'
    ),
    HandleAnswerDecreaseAllowance: o(
        [
            { json: 'allowance', js: 'allowance', typ: '' },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'spender', js: 'spender', typ: '' },
        ],
        'any'
    ),
    HandleAnswerDeposit: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerIncreaseAllowance: o(
        [
            { json: 'allowance', js: 'allowance', typ: '' },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'spender', js: 'spender', typ: '' },
        ],
        'any'
    ),
    HandleAnswerMint: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerRedeem: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerRegisterReceive: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerRemoveMinters: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerRevokePermit: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerSend: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerSendFrom: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerSetContractStatus: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerSetMinters: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerSetViewingKey: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerTransfer: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleAnswerTransferFrom: o(
        [{ json: 'status', js: 'status', typ: r('ResponseStatus') }],
        'any'
    ),
    HandleMsg: o(
        [
            {
                json: 'redeem',
                js: 'redeem',
                typ: u(undefined, r('HandleMsgRedeem')),
            },
            {
                json: 'deposit',
                js: 'deposit',
                typ: u(undefined, r('HandleMsgDeposit')),
            },
            {
                json: 'transfer',
                js: 'transfer',
                typ: u(undefined, r('HandleMsgTransfer')),
            },
            { json: 'send', js: 'send', typ: u(undefined, r('HandleMsgSend')) },
            {
                json: 'batch_transfer',
                js: 'batch_transfer',
                typ: u(undefined, r('HandleMsgBatchTransfer')),
            },
            {
                json: 'batch_send',
                js: 'batch_send',
                typ: u(undefined, r('HandleMsgBatchSend')),
            },
            { json: 'burn', js: 'burn', typ: u(undefined, r('HandleMsgBurn')) },
            {
                json: 'register_receive',
                js: 'register_receive',
                typ: u(undefined, r('HandleMsgRegisterReceive')),
            },
            {
                json: 'create_viewing_key',
                js: 'create_viewing_key',
                typ: u(undefined, r('HandleMsgCreateViewingKey')),
            },
            {
                json: 'set_viewing_key',
                js: 'set_viewing_key',
                typ: u(undefined, r('HandleMsgSetViewingKey')),
            },
            {
                json: 'increase_allowance',
                js: 'increase_allowance',
                typ: u(undefined, r('HandleMsgIncreaseAllowance')),
            },
            {
                json: 'decrease_allowance',
                js: 'decrease_allowance',
                typ: u(undefined, r('HandleMsgDecreaseAllowance')),
            },
            {
                json: 'transfer_from',
                js: 'transfer_from',
                typ: u(undefined, r('HandleMsgTransferFrom')),
            },
            {
                json: 'send_from',
                js: 'send_from',
                typ: u(undefined, r('HandleMsgSendFrom')),
            },
            {
                json: 'batch_transfer_from',
                js: 'batch_transfer_from',
                typ: u(undefined, r('HandleMsgBatchTransferFrom')),
            },
            {
                json: 'batch_send_from',
                js: 'batch_send_from',
                typ: u(undefined, r('HandleMsgBatchSendFrom')),
            },
            {
                json: 'burn_from',
                js: 'burn_from',
                typ: u(undefined, r('HandleMsgBurnFrom')),
            },
            {
                json: 'batch_burn_from',
                js: 'batch_burn_from',
                typ: u(undefined, r('HandleMsgBatchBurnFrom')),
            },
            { json: 'mint', js: 'mint', typ: u(undefined, r('HandleMsgMint')) },
            {
                json: 'batch_mint',
                js: 'batch_mint',
                typ: u(undefined, r('HandleMsgBatchMint')),
            },
            {
                json: 'add_minters',
                js: 'add_minters',
                typ: u(undefined, r('HandleMsgAddMinters')),
            },
            {
                json: 'remove_minters',
                js: 'remove_minters',
                typ: u(undefined, r('HandleMsgRemoveMinters')),
            },
            {
                json: 'set_minters',
                js: 'set_minters',
                typ: u(undefined, r('HandleMsgSetMinters')),
            },
            {
                json: 'change_admin',
                js: 'change_admin',
                typ: u(undefined, r('HandleMsgChangeAdmin')),
            },
            {
                json: 'set_contract_status',
                js: 'set_contract_status',
                typ: u(undefined, r('HandleMsgSetContractStatus')),
            },
            {
                json: 'revoke_permit',
                js: 'revoke_permit',
                typ: u(undefined, r('HandleMsgRevokePermit')),
            },
        ],
        'any'
    ),
    HandleMsgAddMinters: o(
        [
            { json: 'minters', js: 'minters', typ: a('') },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgBatchBurnFrom: o(
        [
            { json: 'actions', js: 'actions', typ: a(r('BurnFromAction')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    BurnFromAction: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'owner', js: 'owner', typ: '' },
        ],
        'any'
    ),
    HandleMsgBatchMint: o(
        [
            { json: 'actions', js: 'actions', typ: a(r('MintAction')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    MintAction: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'recipient', js: 'recipient', typ: '' },
        ],
        'any'
    ),
    HandleMsgBatchSend: o(
        [
            { json: 'actions', js: 'actions', typ: a(r('SendAction')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    SendAction: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'msg', js: 'msg', typ: u(undefined, u(null, '')) },
            { json: 'recipient', js: 'recipient', typ: '' },
            {
                json: 'recipient_code_hash',
                js: 'recipient_code_hash',
                typ: u(undefined, u(null, '')),
            },
        ],
        'any'
    ),
    HandleMsgBatchSendFrom: o(
        [
            { json: 'actions', js: 'actions', typ: a(r('SendFromAction')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    SendFromAction: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'msg', js: 'msg', typ: u(undefined, u(null, '')) },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'recipient', js: 'recipient', typ: '' },
            {
                json: 'recipient_code_hash',
                js: 'recipient_code_hash',
                typ: u(undefined, u(null, '')),
            },
        ],
        'any'
    ),
    HandleMsgBatchTransfer: o(
        [
            { json: 'actions', js: 'actions', typ: a(r('TransferAction')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    TransferAction: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'recipient', js: 'recipient', typ: '' },
        ],
        'any'
    ),
    HandleMsgBatchTransferFrom: o(
        [
            { json: 'actions', js: 'actions', typ: a(r('TransferFromAction')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    TransferFromAction: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'recipient', js: 'recipient', typ: '' },
        ],
        'any'
    ),
    HandleMsgBurn: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgBurnFrom: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgChangeAdmin: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgCreateViewingKey: o(
        [
            { json: 'entropy', js: 'entropy', typ: '' },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgDecreaseAllowance: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            {
                json: 'expiration',
                js: 'expiration',
                typ: u(undefined, u(0, null)),
            },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
            { json: 'spender', js: 'spender', typ: '' },
        ],
        'any'
    ),
    HandleMsgDeposit: o(
        [{ json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) }],
        'any'
    ),
    HandleMsgIncreaseAllowance: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            {
                json: 'expiration',
                js: 'expiration',
                typ: u(undefined, u(0, null)),
            },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
            { json: 'spender', js: 'spender', typ: '' },
        ],
        'any'
    ),
    HandleMsgMint: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
            { json: 'recipient', js: 'recipient', typ: '' },
        ],
        'any'
    ),
    HandleMsgRedeem: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'denom', js: 'denom', typ: u(undefined, u(null, '')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgRegisterReceive: o(
        [
            { json: 'code_hash', js: 'code_hash', typ: '' },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgRemoveMinters: o(
        [
            { json: 'minters', js: 'minters', typ: a('') },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgRevokePermit: o(
        [
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
            { json: 'permit_name', js: 'permit_name', typ: '' },
        ],
        'any'
    ),
    HandleMsgSend: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'msg', js: 'msg', typ: u(undefined, u(null, '')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
            { json: 'recipient', js: 'recipient', typ: '' },
            {
                json: 'recipient_code_hash',
                js: 'recipient_code_hash',
                typ: u(undefined, u(null, '')),
            },
        ],
        'any'
    ),
    HandleMsgSendFrom: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'msg', js: 'msg', typ: u(undefined, u(null, '')) },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
            { json: 'recipient', js: 'recipient', typ: '' },
            {
                json: 'recipient_code_hash',
                js: 'recipient_code_hash',
                typ: u(undefined, u(null, '')),
            },
        ],
        'any'
    ),
    HandleMsgSetContractStatus: o(
        [
            { json: 'level', js: 'level', typ: r('ContractStatusLevel') },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgSetMinters: o(
        [
            { json: 'minters', js: 'minters', typ: a('') },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgSetViewingKey: o(
        [
            { json: 'key', js: 'key', typ: '' },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    HandleMsgTransfer: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
            { json: 'recipient', js: 'recipient', typ: '' },
        ],
        'any'
    ),
    HandleMsgTransferFrom: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'padding', js: 'padding', typ: u(undefined, u(null, '')) },
            { json: 'recipient', js: 'recipient', typ: '' },
        ],
        'any'
    ),
    InitMsg: o(
        [
            { json: 'admin', js: 'admin', typ: u(undefined, u(null, '')) },
            {
                json: 'config',
                js: 'config',
                typ: u(undefined, u(null, r('InitConfig'))),
            },
            { json: 'decimals', js: 'decimals', typ: 0 },
            {
                json: 'initial_balances',
                js: 'initial_balances',
                typ: u(undefined, u(a(r('InitialBalance')), null)),
            },
            { json: 'name', js: 'name', typ: '' },
            { json: 'prng_seed', js: 'prng_seed', typ: '' },
            { json: 'symbol', js: 'symbol', typ: '' },
        ],
        'any'
    ),
    InitConfig: o(
        [
            {
                json: 'enable_burn',
                js: 'enable_burn',
                typ: u(undefined, u(true, null)),
            },
            {
                json: 'enable_deposit',
                js: 'enable_deposit',
                typ: u(undefined, u(true, null)),
            },
            {
                json: 'enable_mint',
                js: 'enable_mint',
                typ: u(undefined, u(true, null)),
            },
            {
                json: 'enable_redeem',
                js: 'enable_redeem',
                typ: u(undefined, u(true, null)),
            },
            {
                json: 'public_total_supply',
                js: 'public_total_supply',
                typ: u(undefined, u(true, null)),
            },
        ],
        'any'
    ),
    InitialBalance: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'amount', js: 'amount', typ: '' },
        ],
        'any'
    ),
    QueryAnswer: o(
        [
            {
                json: 'token_info',
                js: 'token_info',
                typ: u(undefined, r('TokenInfo')),
            },
            {
                json: 'token_config',
                js: 'token_config',
                typ: u(undefined, r('TokenConfig')),
            },
            {
                json: 'contract_status',
                js: 'contract_status',
                typ: u(undefined, r('ContractStatus')),
            },
            {
                json: 'exchange_rate',
                js: 'exchange_rate',
                typ: u(undefined, r('ExchangeRate')),
            },
            {
                json: 'allowance',
                js: 'allowance',
                typ: u(undefined, r('QueryAnswerAllowance')),
            },
            {
                json: 'balance',
                js: 'balance',
                typ: u(undefined, r('QueryAnswerBalance')),
            },
            {
                json: 'transfer_history',
                js: 'transfer_history',
                typ: u(undefined, r('QueryAnswerTransferHistory')),
            },
            {
                json: 'transaction_history',
                js: 'transaction_history',
                typ: u(undefined, r('QueryAnswerTransactionHistory')),
            },
            {
                json: 'viewing_key_error',
                js: 'viewing_key_error',
                typ: u(undefined, r('ViewingKeyError')),
            },
            { json: 'minters', js: 'minters', typ: u(undefined, r('Minters')) },
        ],
        'any'
    ),
    QueryAnswerAllowance: o(
        [
            { json: 'allowance', js: 'allowance', typ: '' },
            {
                json: 'expiration',
                js: 'expiration',
                typ: u(undefined, u(0, null)),
            },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'spender', js: 'spender', typ: '' },
        ],
        'any'
    ),
    QueryAnswerBalance: o([{ json: 'amount', js: 'amount', typ: '' }], 'any'),
    ContractStatus: o(
        [{ json: 'status', js: 'status', typ: r('ContractStatusLevel') }],
        'any'
    ),
    ExchangeRate: o(
        [
            { json: 'denom', js: 'denom', typ: '' },
            { json: 'rate', js: 'rate', typ: '' },
        ],
        'any'
    ),
    Minters: o([{ json: 'minters', js: 'minters', typ: a('') }], 'any'),
    TokenConfig: o(
        [
            { json: 'burn_enabled', js: 'burn_enabled', typ: true },
            { json: 'deposit_enabled', js: 'deposit_enabled', typ: true },
            { json: 'mint_enabled', js: 'mint_enabled', typ: true },
            {
                json: 'public_total_supply',
                js: 'public_total_supply',
                typ: true,
            },
            { json: 'redeem_enabled', js: 'redeem_enabled', typ: true },
        ],
        'any'
    ),
    TokenInfo: o(
        [
            { json: 'decimals', js: 'decimals', typ: 0 },
            { json: 'name', js: 'name', typ: '' },
            { json: 'symbol', js: 'symbol', typ: '' },
            {
                json: 'total_supply',
                js: 'total_supply',
                typ: u(undefined, u(null, '')),
            },
        ],
        'any'
    ),
    QueryAnswerTransactionHistory: o(
        [
            { json: 'total', js: 'total', typ: u(undefined, u(0, null)) },
            { json: 'txs', js: 'txs', typ: a(r('RichTx')) },
        ],
        'any'
    ),
    RichTx: o(
        [
            { json: 'action', js: 'action', typ: r('TxAction') },
            { json: 'block_height', js: 'block_height', typ: 0 },
            { json: 'block_time', js: 'block_time', typ: 0 },
            { json: 'coins', js: 'coins', typ: r('Coin') },
            { json: 'id', js: 'id', typ: 0 },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
        ],
        'any'
    ),
    TxAction: o(
        [
            {
                json: 'transfer',
                js: 'transfer',
                typ: u(undefined, r('TxActionTransfer')),
            },
            { json: 'mint', js: 'mint', typ: u(undefined, r('TxActionMint')) },
            { json: 'burn', js: 'burn', typ: u(undefined, r('TxActionBurn')) },
            { json: 'deposit', js: 'deposit', typ: u(undefined, m('any')) },
            { json: 'redeem', js: 'redeem', typ: u(undefined, m('any')) },
        ],
        'any'
    ),
    TxActionBurn: o(
        [
            { json: 'burner', js: 'burner', typ: '' },
            { json: 'owner', js: 'owner', typ: '' },
        ],
        'any'
    ),
    TxActionMint: o(
        [
            { json: 'minter', js: 'minter', typ: '' },
            { json: 'recipient', js: 'recipient', typ: '' },
        ],
        'any'
    ),
    TxActionTransfer: o(
        [
            { json: 'from', js: 'from', typ: '' },
            { json: 'recipient', js: 'recipient', typ: '' },
            { json: 'sender', js: 'sender', typ: '' },
        ],
        'any'
    ),
    Coin: o(
        [
            { json: 'amount', js: 'amount', typ: '' },
            { json: 'denom', js: 'denom', typ: '' },
        ],
        'any'
    ),
    QueryAnswerTransferHistory: o(
        [
            { json: 'total', js: 'total', typ: u(undefined, u(0, null)) },
            { json: 'txs', js: 'txs', typ: a(r('Tx')) },
        ],
        'any'
    ),
    Tx: o(
        [
            {
                json: 'block_height',
                js: 'block_height',
                typ: u(undefined, u(0, null)),
            },
            {
                json: 'block_time',
                js: 'block_time',
                typ: u(undefined, u(0, null)),
            },
            { json: 'coins', js: 'coins', typ: r('Coin') },
            { json: 'from', js: 'from', typ: '' },
            { json: 'id', js: 'id', typ: 0 },
            { json: 'memo', js: 'memo', typ: u(undefined, u(null, '')) },
            { json: 'receiver', js: 'receiver', typ: '' },
            { json: 'sender', js: 'sender', typ: '' },
        ],
        'any'
    ),
    ViewingKeyError: o([{ json: 'msg', js: 'msg', typ: '' }], 'any'),
    QueryMsg: o(
        [
            {
                json: 'token_info',
                js: 'token_info',
                typ: u(undefined, m('any')),
            },
            {
                json: 'token_config',
                js: 'token_config',
                typ: u(undefined, m('any')),
            },
            {
                json: 'contract_status',
                js: 'contract_status',
                typ: u(undefined, m('any')),
            },
            {
                json: 'exchange_rate',
                js: 'exchange_rate',
                typ: u(undefined, m('any')),
            },
            {
                json: 'allowance',
                js: 'allowance',
                typ: u(undefined, r('QueryMsgAllowance')),
            },
            {
                json: 'balance',
                js: 'balance',
                typ: u(undefined, r('QueryMsgBalance')),
            },
            {
                json: 'transfer_history',
                js: 'transfer_history',
                typ: u(undefined, r('QueryMsgTransferHistory')),
            },
            {
                json: 'transaction_history',
                js: 'transaction_history',
                typ: u(undefined, r('QueryMsgTransactionHistory')),
            },
            { json: 'minters', js: 'minters', typ: u(undefined, m('any')) },
            {
                json: 'with_permit',
                js: 'with_permit',
                typ: u(undefined, r('WithPermit')),
            },
        ],
        'any'
    ),
    QueryMsgAllowance: o(
        [
            { json: 'key', js: 'key', typ: '' },
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'spender', js: 'spender', typ: '' },
        ],
        'any'
    ),
    QueryMsgBalance: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'key', js: 'key', typ: '' },
        ],
        'any'
    ),
    QueryMsgTransactionHistory: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'key', js: 'key', typ: '' },
            { json: 'page', js: 'page', typ: u(undefined, u(0, null)) },
            { json: 'page_size', js: 'page_size', typ: 0 },
        ],
        'any'
    ),
    QueryMsgTransferHistory: o(
        [
            { json: 'address', js: 'address', typ: '' },
            { json: 'key', js: 'key', typ: '' },
            { json: 'page', js: 'page', typ: u(undefined, u(0, null)) },
            { json: 'page_size', js: 'page_size', typ: 0 },
        ],
        'any'
    ),
    WithPermit: o(
        [
            {
                json: 'permit',
                js: 'permit',
                typ: r('PermitForTokenPermissions'),
            },
            { json: 'query', js: 'query', typ: r('QueryWithPermit') },
        ],
        'any'
    ),
    PermitForTokenPermissions: o(
        [
            {
                json: 'params',
                js: 'params',
                typ: r('PermitParamsForTokenPermissions'),
            },
            { json: 'signature', js: 'signature', typ: r('PermitSignature') },
        ],
        'any'
    ),
    PermitParamsForTokenPermissions: o(
        [
            { json: 'allowed_tokens', js: 'allowed_tokens', typ: a('') },
            { json: 'chain_id', js: 'chain_id', typ: '' },
            {
                json: 'permissions',
                js: 'permissions',
                typ: a(r('TokenPermissions')),
            },
            { json: 'permit_name', js: 'permit_name', typ: '' },
        ],
        'any'
    ),
    PermitSignature: o(
        [
            { json: 'pub_key', js: 'pub_key', typ: r('PubKey') },
            { json: 'signature', js: 'signature', typ: '' },
        ],
        'any'
    ),
    PubKey: o(
        [
            { json: 'type', js: 'type', typ: '' },
            { json: 'value', js: 'value', typ: '' },
        ],
        'any'
    ),
    QueryWithPermit: o(
        [
            {
                json: 'allowance',
                js: 'allowance',
                typ: u(undefined, r('QueryWithPermitAllowance')),
            },
            { json: 'balance', js: 'balance', typ: u(undefined, m('any')) },
            {
                json: 'transfer_history',
                js: 'transfer_history',
                typ: u(undefined, r('QueryWithPermitTransferHistory')),
            },
            {
                json: 'transaction_history',
                js: 'transaction_history',
                typ: u(undefined, r('QueryWithPermitTransactionHistory')),
            },
        ],
        'any'
    ),
    QueryWithPermitAllowance: o(
        [
            { json: 'owner', js: 'owner', typ: '' },
            { json: 'spender', js: 'spender', typ: '' },
        ],
        'any'
    ),
    QueryWithPermitTransactionHistory: o(
        [
            { json: 'page', js: 'page', typ: u(undefined, u(0, null)) },
            { json: 'page_size', js: 'page_size', typ: 0 },
        ],
        'any'
    ),
    QueryWithPermitTransferHistory: o(
        [
            { json: 'page', js: 'page', typ: u(undefined, u(0, null)) },
            { json: 'page_size', js: 'page_size', typ: 0 },
        ],
        'any'
    ),
    ResponseStatus: ['failure', 'success'],
    ContractStatusLevel: ['normal_run', 'stop_all', 'stop_all_but_redeems'],
    TokenPermissions: ['allowance', 'balance', 'history', 'owner'],
};
