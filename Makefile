GIT_COMMIT_HASH = $(shell git rev-parse --short HEAD)
LOCAL_DEV_MNEMONIC_A = grant rice replace explain federal release fix clever romance raise often wild taxi quarter soccer fiber love must tape steak together observe swap guitar
LOCAL_DEV_MNEMONIC_B = jelly shadow frog dirt dragon use armed praise universe win jungle close inmate rain oil canvas beauty pioneer chef soccer icon dizzy thunder meadow
LOCAL_CHAIN_ID = secretdev-1

submodule:
	git submodule update --init

test-token: submodule
	cd contracts/token \
	&& make test

test-bitcoin-spv: 
	cd contracts/bitcoin_spv \
	&& make test

test-gateway:
	cd contracts/gateway \
	&& make test

test-sfps:
	cd contracts/sfps \
	&& make test

test-log:
	cd contracts/log \
	&& make test

test-treasury:
	cd contracts/treasury \
	&& make test

test-shuriken:
	cd contracts/shuriken \
	&& make test

test-multisig:
	cd contracts/multisig \
	&& make test

test-finance-admin-v1:
	cd contracts/finance_admin_v1 \
	&& make test

test-sfps-lib:
	cd contracts/libs/sfps_lib \
	&& cargo test

test: test-token test-bitcoin-spv test-gateway test-sfps test-log test-treasury test-shuriken test-multisig test-finance-admin-v1 test-sfps-lib

compile-token: submodule
	cd contracts/token \
	&& cargo check \
	&& make compile-optimized

compile-bitcoin-spv:
	cd contracts/bitcoin_spv \
	&& cargo check \
	&& make compile-optimized

compile-gateway:
	cd contracts/gateway \
	&& cargo check \
	&& make compile-optimized

compile-sfps:
	cd contracts/sfps \
	&& cargo check \
	&& make compile-optimized

compile-sfps-full-signatures-test:
	cd contracts/sfps \
	&& cargo check \
	&& make compile-optimized-full-signatures-test

compile-log:
	cd contracts/log \
	&& cargo check \
	&& make compile-optimized

compile-treasury:
	cd contracts/treasury \
	&& cargo check \
	&& make compile-optimized

compile-shuriken:
	cd contracts/shuriken \
	&& cargo check \
	&& make compile-optimized

compile-multisig:
	cd contracts/multisig \
	&& cargo check \
	&& make compile-optimized

compile-finance-admin-v1:
	cd contracts/finance_admin_v1 \
	&& cargo check \
	&& make compile-optimized

compile: compile-token compile-bitcoin-spv compile-gateway compile-sfps compile-log compile-treasury compile-shuriken compile-multisig compile-finance-admin-v1

compile-full-signatures-test: compile-token compile-bitcoin-spv compile-gateway compile-sfps-full-signatures-test compile-log compile-treasury compile-shuriken compile-multisig compile-finance-admin-v1

schema-token: submodule
	cd contracts/token \
	&& make schema

schema-bitcoin-spv:
	cd contracts/bitcoin_spv \
	&& make schema

schema-gateway:
	cd contracts/gateway \
	&& make schema

schema-sfps:
	cd contracts/sfps \
	&& make schema

schema-log:
	cd contracts/log \
	&& make schema

schema-treasury:
	cd contracts/treasury \
	&& make schema

schema-shuriken:
	cd contracts/shuriken \
	&& make schema

schema-multisig:
	cd contracts/multisig \
	&& make schema

schema-finance-admin-v1:
	cd contracts/finance_admin_v1 \
	&& make schema

schema: schema-token schema-bitcoin-spv schema-gateway schema-sfps schema-log schema-treasury schema-shuriken schema-multisig schema-finance-admin-v1

start-local:
	docker-compose -f docker-compose.yml -p sbtc_local up --force-recreate --build

deploy-local: compile-full-signatures-test
	cd deploy \
	&& yarn \
	&& TENDERMINT_RPC_URL=http://localhost:26657 \
	GRPC_WEB_URL=http://localhost:9091 \
	MNEMONIC='$(LOCAL_DEV_MNEMONIC_A)' \
	ENVIRONMENT=local \
	GIT_COMMIT_HASH=$(GIT_COMMIT_HASH) \
	CHAIN_ID=$(LOCAL_CHAIN_ID) \
	TRANSACTION_WAIT_TIME=0 \
	yarn dev

build-sbtc-js: schema
	cd client/sbtc-js \
	&& yarn \
	&& yarn schema \
	&& yarn build \
	&& yarn link

build-shuriken-node: build-sbtc-js 
	cd client/shuriken-node \
	&& yarn \
	&& yarn build \
	&& yarn link

integration-test: build-sbtc-js build-shuriken-node
	cd integration-test \
	&& yarn \
	&& REGTEST_SERVER_URL=http://localhost:8080/1 \
	GRPC_WEB_URL=http://localhost:9091 \
	TENDERMINT_RPC_URL=http://localhost:26657 \
	MNEMONIC='$(LOCAL_DEV_MNEMONIC_A)' \
	FEE_REPORT_FILE_PATH='../FeeReport.md' \
	GATEWAY_ADDRESS=secret1y45vkh0n6kplaeqw6ratuertapxupz532vxnn3 \
	SNB_ADDRESS=secret1xzlgeyuuyqje79ma6vllregprkmgwgavk8y798 \
	TREASURY_ADDRESS=secret15rrl3qjafxzlzguu5x29xh29pam35uetwpfnna \
	SHURIKEN_ADDRESS=secret17ak0ku2uvfs04w4u867xhgvfg4ta6mgqdg8j4q \
	FINANCE_ADMIN_ADDRESS=secret1uul3yzm2lgskp3dxpj0zg558hppxk6ptyljer5 \
	LOG_ADDRESS=secret10pyejy66429refv3g35g2t7am0was7ya6hvrzf \
	RECEIVE_ADDRESS=bcrt1q2wurym9pfslr9mtrl5q579z44cfxw4qnduu604 \
	CHAIN_ID=$(LOCAL_CHAIN_ID) \
	yarn dev

shuriken-node-local: build-shuriken-node
	cd client/shuriken-node \
	&& GRPC_WEB_URL=http://localhost:9091 \
	CHAIN_ID=$(LOCAL_CHAIN_ID) \
	MNEMONIC='$(LOCAL_DEV_MNEMONIC_B)' \
	SHURIKEN_ADDRESS=secret17ak0ku2uvfs04w4u867xhgvfg4ta6mgqdg8j4q \
	BITCOIN_API_TYPE=regtest_server \
	BITCOIN_REGTEST_SERVER_URL=http://localhost:8080/1 \
	TENDERMINT_RPC_URL=http://localhost:26657 \
	yarn dev

frontend-local:  build-sbtc-js
	cd client/frontend \
	&& NEXT_PUBLIC_GATEWAY_ADDRESS=secret1xzlgeyuuyqje79ma6vllregprkmgwgavk8y798 \
	NEXT_PUBLIC_SN_NETWORK=enigma-pub-testnet-3 \
	yarn dev