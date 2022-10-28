import {
    EncryptionUtils,
    EncryptionUtilsImpl,
    SecretNetworkClient,
} from 'secretjs';
import { Signer } from 'secretjs/dist/wallet_amino';
import { ServiceClientImpl } from 'secretjs/dist/protobuf_stuff/cosmos/tx/v1beta1/service';
import { GrpcWebImpl } from 'secretjs/dist/protobuf_stuff/cosmos/tx/v1beta1/service';
import { NodeHttpTransport } from '@improbable-eng/grpc-web-node-http-transport';

export default class ShinobiClient {
    constructor(
        public readonly grpcWebUrl: string,
        public readonly chainId: string,
        public readonly encryptionUtils: EncryptionUtils,
        public readonly txService: ServiceClientImpl,
        public readonly sn: SecretNetworkClient
    ) { }

    public static async create(
        grpcWebUrl: string,
        chainId: string,
        wallet: Signer,
        walletAddress: string,
        encryptionUtils?: EncryptionUtils
    ): Promise<ShinobiClient> {
        if (!encryptionUtils) {
            const querier = (
                await SecretNetworkClient.create({
                    grpcWebUrl,
                    chainId,
                })
            ).query;
            encryptionUtils = new EncryptionUtilsImpl(
                querier.registration,
                undefined,
                chainId
            );
        }
        const secretNetworkClient = await SecretNetworkClient.create({
            grpcWebUrl,
            chainId,
            wallet,
            walletAddress,
            encryptionUtils,
        });
        const grpcWeb = new GrpcWebImpl(grpcWebUrl, {
            transport: NodeHttpTransport(),
        });
        const txService = new ServiceClientImpl(grpcWeb);
        return new ShinobiClient(
            grpcWebUrl,
            chainId,
            encryptionUtils,
            txService,
            secretNetworkClient
        );
    }
}
