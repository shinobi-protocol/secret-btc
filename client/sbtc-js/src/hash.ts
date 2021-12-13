import { crypto } from 'bitcoinjs-lib';
export function sha256d(buf: Buffer): Buffer {
    return crypto.sha256(crypto.sha256(buf));
}

export function sha256(buf: Buffer): Buffer {
    return crypto.sha256(buf);
}
