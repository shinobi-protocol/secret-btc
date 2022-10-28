import * as _m0 from 'protobufjs/minimal';

export interface Encoder<M> {
    encode(message: M, writer?: _m0.Writer): _m0.Writer;
}

export function encodeToBase64<M>(encoder: Encoder<M>, message: M): string {
    const buffer = encodeToBuffer(encoder, message);
    return buffer.toString('base64');
}

export function encodeToBuffer<M>(encoder: Encoder<M>, message: M): Buffer {
    return Buffer.from(encoder.encode(message).finish());
}
