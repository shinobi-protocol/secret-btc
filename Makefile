GIT_COMMIT_HASH = $(shell git rev-parse --short HEAD)

LOCAL_DEV_MNEMONIC_A = grant rice replace explain federal release fix clever romance raise often wild taxi quarter soccer fiber love must tape steak together observe swap guitar
LOCAL_DEV_MNEMONIC_B = jelly shadow frog dirt dragon use armed praise universe win jungle close inmate rain oil canvas beauty pioneer chef soccer icon dizzy thunder meadow

LOCAL_CHAIN_ID = secretdev-1
TESTNET_CHAIN_ID = pulsar-2
MAINNET_CHAIN_ID = secret-4

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


test-shuriken:
	cd contracts/shuriken \
	&& make test

test-multisig:
	cd contracts/multisig \
	&& make test

test-vesting:
	cd contracts/libs/vesting \
	&& cargo test

test-sfps-lib:
	cd contracts/libs/sfps_lib \
	&& cargo test

test-state:
	cd contracts/state \
	&& cargo test

test: test-token test-bitcoin-spv test-gateway test-sfps test-log test-shuriken test-multisig test-sfps-lib test-vesting test-state

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

compile-shuriken:
	cd contracts/shuriken \
	&& cargo check \
	&& make compile-optimized

compile-multisig:
	cd contracts/multisig \
	&& cargo check \
	&& make compile-optimized


compile-state:
	cd contracts/state \
	&& cargo check \
	&& make compile-optimized

compile-vesting:
	cd contracts/vesting \
	&& cargo check \
	&& make compile-optimized

compile: compile-token compile-bitcoin-spv compile-gateway compile-sfps compile-log compile-shuriken compile-multisig compile-vesting compile-state

compile-full-signatures-test: compile-token compile-bitcoin-spv compile-gateway compile-sfps-full-signatures-test compile-log compile-shuriken compile-multisig compile-vesting compile-state

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

schema-shuriken:
	cd contracts/shuriken \
	&& make schema

schema-multisig:
	cd contracts/multisig \
	&& make schema


schema-state:
	cd contracts/state \
	&& make schema

schema-vesting:
	cd contracts/vesting \
	&& make schema

schema: schema-token schema-bitcoin-spv schema-gateway schema-sfps schema-log schema-shuriken schema-multisig schema-state schema-vesting

start-local:
	docker-compose -f docker-compose.yml -p sbtc_local up --force-recreate --build

deploy-local: build-sbtc-js compile-full-signatures-test
	cd deploy \
	&& yarn \
	&& GRPC_WEB_URL=http://localhost:9091 \
	LCD_URL=http://localhost:1317 \
	MNEMONIC='$(LOCAL_DEV_MNEMONIC_A)' \
	ENVIRONMENT=local \
	GIT_COMMIT_HASH=$(GIT_COMMIT_HASH) \
	CHAIN_ID=$(LOCAL_CHAIN_ID) \
	TRANSACTION_WAIT_TIME=0 \
	yarn dev

deploy-testnet: compile
	cd deploy \
	&& yarn \
	&& GRPC_WEB_URL=https://pulsar-2.api.trivium.network:9091 \
	LCD_URL=https://pulsar-2.api.trivium.network:1317 \
	MNEMONIC='$(MNEMONIC)' \
	ENVIRONMENT=testnet \
	GIT_COMMIT_HASH=$(GIT_COMMIT_HASH) \
	CHAIN_ID=$(TESTNET_CHAIN_ID) \
	TRANSACTION_WAIT_TIME=0 \
	yarn dev

deploy-mainnet-test: compile
	cd deploy \
	&& yarn \
	&& GRPC_WEB_URL=https://secret-4.api.trivium.network:9091 \
	LCD_URL=https://secret-4.api.trivium.network:1317 \
	MNEMONIC='$(MNEMONIC)' \
	ENVIRONMENT=mainnet-test \
	GIT_COMMIT_HASH=$(GIT_COMMIT_HASH) \
	CHAIN_ID=$(MAINNET_CHAIN_ID) \
	TRANSACTION_WAIT_TIME=5 \
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
	MNEMONIC='$(LOCAL_DEV_MNEMONIC_A)' \
	FEE_REPORT_FILE_PATH='../FeeReport.md' \
	GATEWAY_ADDRESS=secret15rrl3qjafxzlzguu5x29xh29pam35uetwpfnna \
	SNB_ADDRESS=secret1y45vkh0n6kplaeqw6ratuertapxupz532vxnn3 \
	SHURIKEN_ADDRESS=secret17ak0ku2uvfs04w4u867xhgvfg4ta6mgqdg8j4q \
	LOG_ADDRESS=secret1sh36qn08g4cqg685cfzmyxqv2952q6r8vqktuh \
	VESTING_ADDRESS=secret1uul3yzm2lgskp3dxpj0zg558hppxk6ptyljer5 \
	RECEIVE_ADDRESS=bcrt1q2wurym9pfslr9mtrl5q579z44cfxw4qnduu604 \
	CHAIN_ID=$(LOCAL_CHAIN_ID) \
	yarn dev

shuriken-node-local: build-shuriken-node
	cd client/shuriken-node \
	&& GRPC_WEB_URL=http://localhost:9091 \
	CHAIN_ID=$(LOCAL_CHAIN_ID) \
	MNEMONIC='$(LOCAL_DEV_MNEMONIC_B)' \
	SHURIKEN_ADDRESS=secret15rrl3qjafxzlzguu5x29xh29pam35uetwpfnna \
	BITCOIN_API_TYPE=regtest_server \
	BITCOIN_REGTEST_SERVER_URL=http://localhost:8080/1 \
	SFPS_BLOCK_PER_TX=10 \
	yarn dev

install-frontend: build-sbtc-js
	cd client/frontend \
	&& yarn


frontend-local: build-sbtc-js
	cd client/frontend \
	yarn \
	&& NEXT_PUBLIC_GATEWAY_ADDRESS=secret1y45vkh0n6kplaeqw6ratuertapxupz532vxnn3 \
	NEXT_PUBLIC_SNB_ADDRESS=secret1xzlgeyuuyqje79ma6vllregprkmgwgavk8y798 \
	NEXT_PUBLIC_TREASURY_ADDRESS=secret15rrl3qjafxzlzguu5x29xh29pam35uetwpfnna \
	NEXT_PUBLIC_LOG_ADDRESS=secret10pyejy66429refv3g35g2t7am0was7ya6hvrzf \
	NEXT_PUBLIC_SN_NETWORK=secretdev-1 \
	NEXT_PUBLIC_BITCOIN_CONFIRMATION=6 \
	NEXT_PUBLIC_ENABLE_BASIC_AUTH=false \
	yarn dev


frontend-pulsar-2: build-sbtc-js
	cd client/frontend \
	yarn \
	&& NEXT_PUBLIC_GATEWAY_ADDRESS=secret1zg3jgj4kqw9tp6lgs9fkvc4wj2fwx6srpqjxyp \
	NEXT_PUBLIC_SNB_ADDRESS=secret1nlpd7ak7kfsgvuz6gnqhzp79vuyqpdjlsm2jpg \
	NEXT_PUBLIC_LOG_ADDRESS=secret1cztpnqmq6pwxa4w6777dxr0d0ztvmhtecd7c4y \
	NEXT_PUBLIC_SN_NETWORK=pulsar-2 \
	NEXT_PUBLIC_BITCOIN_CONFIRMATION=1 \
	NEXT_PUBLIC_ENABLE_BASIC_AUTH=false \
	yarn dev

frontend-mainnet: build-sbtc-js
	cd client/frontend \
	yarn \
	&& NEXT_PUBLIC_GATEWAY_ADDRESS=secret1fv7trst8ev259xce2vnm7hpk796hymp3lctc57 \
	NEXT_PUBLIC_SNB_ADDRESS=secret1c4zq752dexr0dplnsv55ct0ul9k6jtzne8jrff \
	NEXT_PUBLIC_LOG_ADDRESS=secret1r0hgkflykycxra2l70xtp37aljqm8uc7ucjqt0 \
	NEXT_PUBLIC_SN_NETWORK=secret-4 \
	NEXT_PUBLIC_BITCOIN_CONFIRMATION=1 \
	NEXT_PUBLIC_ENABLE_BASIC_AUTH=false \
	yarn dev