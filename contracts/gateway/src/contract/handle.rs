use crate::contract::query_bitcoin_network::query_bitcoin_network;
use crate::error::Error;
use crate::state::bitcoin_utxo::gen_request_key;
use crate::state::bitcoin_utxo::{
    read_release_request_utxo, write_release_request_utxo, Utxo, UtxoQueue,
};
use crate::state::config::{read_config, write_config};
use crate::state::mint_key::{read_mint_key, remove_mint_key, write_mint_key};
use crate::state::prefix::{PREFIX_PRNG, PREFIX_VIEW_KEY};
use crate::state::suspension_switch::set_suspension_switch;
use crate::state::suspension_switch::suspension_switch;
use bitcoin::blockdata::opcodes::all;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::transaction::{SigHashType, Transaction, TxIn, TxOut};
use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::secp256k1::{sign, Message, SecretKey};
use bitcoin::util::address::{Address, Payload};
use bitcoin::util::sighash::SigHashCache;
use bitcoin::VarInt;
use bitcoin::{OutPoint, PrivateKey, PublicKey, Script};
use cosmwasm_std::HumanAddr;
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdResult, Storage,
};
use secret_toolkit::snip20;
use secret_toolkit::utils::padding::pad_handle_result;
use secret_toolkit::utils::{HandleCallback, Query};
use shared_types::gateway::*;
use shared_types::prng::update_prng;
use shared_types::{
    bitcoin_spv, finance_admin, log, sfps, viewing_key, ContractReference, BLOCK_SIZE,
};
use std::str::FromStr;
use std::string::ToString;

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let suspension_switch = suspension_switch(&deps.storage)?;
    let result = match msg {
        HandleMsg::CreateViewingKey { entropy, .. } => try_create_key(deps, env, entropy),
        HandleMsg::SetViewingKey { key, .. } => try_set_key(deps, env, key),
        HandleMsg::RequestMintAddress { entropy, .. } => {
            if suspension_switch.request_mint_address {
                return Err(Error::contract_err("request mint address is being suspended").into());
            }
            try_request_mint_address(deps, env, entropy)
        }
        HandleMsg::VerifyMintTx {
            height,
            tx,
            merkle_proof,
            ..
        } => {
            if suspension_switch.verify_mint_tx {
                return Err(Error::contract_err("verify mint tx is being suspended").into());
            }
            try_verify_mint_tx(deps, env, height, tx, merkle_proof)
        }
        HandleMsg::ReleaseIncorrectAmountBTC {
            height,
            tx,
            merkle_proof,
            recipient_address,
            fee_per_vb,
            ..
        } => {
            if suspension_switch.release_incorrect_amount_btc {
                return Err(
                    Error::contract_err("release incorrect amount btc is being suspended").into(),
                );
            }
            try_release_incorrect_amount_btc(
                deps,
                env,
                height,
                tx,
                merkle_proof,
                recipient_address,
                fee_per_vb,
            )
        }
        HandleMsg::RequestReleaseBtc {
            entropy, amount, ..
        } => {
            if suspension_switch.request_release_btc {
                return Err(Error::contract_err("request release btc is being suspended").into());
            }
            try_request_release_btc(deps, env, amount, entropy)
        }
        HandleMsg::ClaimReleasedBtc {
            tx_result_proof,
            header_hash_index,
            encryption_key,
            recipient_address,
            fee_per_vb,
            ..
        } => {
            if suspension_switch.claim_release_btc {
                return Err(Error::contract_err("claim release btc is being suspended").into());
            }
            try_claim_released_btc(
                deps,
                env,
                tx_result_proof,
                header_hash_index,
                encryption_key,
                recipient_address,
                fee_per_vb,
            )
        }
        HandleMsg::ChangeFinanceAdmin { new_finance_admin } => {
            try_change_finance_admin(deps, env, new_finance_admin)
        }
        HandleMsg::ChangeOwner { new_owner } => try_change_owner(deps, env, new_owner),
        HandleMsg::SetSuspensionSwitch { suspension_switch } => {
            try_set_suspension_switch(deps, env, suspension_switch)
        }
        HandleMsg::ReleaseBtcByOwner {
            tx_value,
            max_input_length,
            recipient_address,
            fee_per_vb,
        } => try_release_btc_by_owner(
            deps,
            env,
            tx_value,
            max_input_length,
            recipient_address,
            fee_per_vb,
        ),
    };
    pad_handle_result(result.map_err(|e| e.into()), BLOCK_SIZE)
}

/// Same logic as SNIP20 Viewing key
fn try_create_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> Result<HandleResponse, Error> {
    let mut rng = update_prng(
        &mut deps.storage,
        PREFIX_PRNG,
        &deps.api.canonical_address(&env.message.sender)?,
        entropy.as_bytes(),
    )?;

    let key = viewing_key::ViewingKey::new(&mut rng);

    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    let mut store =
        viewing_key::ViewingKeyHashStore::from_storage(&mut deps.storage, PREFIX_VIEW_KEY);
    store.write(&message_sender, &key.hash());

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key: key })?),
    })
}

/// Same logic as SNIP20 Viewing key
fn try_set_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    key: viewing_key::ViewingKey,
) -> Result<HandleResponse, Error> {
    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    let mut store =
        viewing_key::ViewingKeyHashStore::from_storage(&mut deps.storage, PREFIX_VIEW_KEY);
    store.write(&message_sender, &key.hash());

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None,
    })
}

fn try_request_mint_address<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: Binary,
) -> Result<HandleResponse, Error> {
    let config = read_config(&deps.storage, &deps.api)?;
    let network = query_bitcoin_network(&deps.querier, config.bitcoin_spv)?;

    let mintor = deps.api.canonical_address(&env.message.sender)?;
    let mut rng = update_prng(&mut deps.storage, PREFIX_PRNG, &mintor, entropy.as_slice())?;
    let mint_key = PrivateKey {
        key: SecretKey::random(&mut rng),
        compressed: true,
        network,
    };
    write_mint_key(&mut deps.storage, &mintor, &mint_key);

    let address = Address::p2wpkh(&mint_key.public_key(), mint_key.network)?.to_string();
    Ok(HandleResponse {
        messages: vec![log::HandleMsg::AddEvents {
            events: vec![(
                env.message.sender,
                log::Event::MintStarted(log::event::MintStartedData {
                    time: env.block.time,
                    address: address.clone(),
                }),
            )],
        }
        .to_cosmos_msg(config.log.hash, config.log.address, None)?],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RequestMintAddress {
            mint_address: address,
        })?),
    })
}

fn extract_vout(outputs: &[TxOut], address: &Address) -> Result<u32, Error> {
    for (i, output) in outputs.iter().enumerate() {
        let payload = Payload::from_script(&output.script_pubkey)
            .ok_or_else(|| Error::contract_err("failed to generate payload"))?;
        if payload == address.payload {
            return Ok(i as u32);
        }
    }
    Err(Error::contract_err("no valid tx output"))
}

fn try_verify_mint_tx<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    height: u32,
    tx: Binary,
    merkle_proof: bitcoin_spv::MerkleProofMsg,
) -> Result<HandleResponse, Error> {
    let mintor = deps.api.canonical_address(&env.message.sender)?;
    let config = read_config(&deps.storage, &deps.api)?;

    //
    // Validate Tx Confirmed
    //
    if let bitcoin_spv::QueryAnswer::VerifyMerkleProof { success } =
        (bitcoin_spv::QueryMsg::VerifyMerkleProof {
            height,
            tx: tx.clone(),
            merkle_proof,
        })
        .query(
            &deps.querier,
            config.bitcoin_spv.hash.clone(),
            config.bitcoin_spv.address.clone(),
        )?
    {
        if success {
            Ok(())
        } else {
            Err(Error::contract_err("merkle proof verification failed"))
        }
    } else {
        Err(Error::contract_err(
            "unexpected query answer from bitcoin spv",
        ))
    }?;

    let network = query_bitcoin_network(&deps.querier, config.bitcoin_spv)?;
    let tx: Transaction = deserialize::<Transaction>(tx.as_slice())?;
    let txid = tx.txid();

    //
    // Validate Tx has an Output destination and value are correct
    //
    let priv_key = read_mint_key(&deps.storage, &mintor, network)?
        .ok_or_else(|| Error::contract_err("message sender does not have mint address"))?;
    let bitcoin_address = Address::p2wpkh(&priv_key.public_key(), priv_key.network)?;
    remove_mint_key(&mut deps.storage, &mintor);
    let vout = extract_vout(&tx.output, &bitcoin_address)?;
    let amount = tx.output[vout as usize].value;
    if !config.btc_tx_values.contains(&amount) {
        return Err(Error::contract_err("sent value is incorrect"));
    }
    //
    // Confirm Mint
    //
    // store utxo
    let mut utxo_queue = UtxoQueue::from_storage(&mut deps.storage, amount);
    utxo_queue.enqueue(Utxo {
        txid,
        vout,
        key: priv_key.key.serialize(),
    })?;
    let sbtc_total_supply = snip20::token_info_query(
        &deps.querier,
        BLOCK_SIZE,
        config.sbtc.hash.clone(),
        config.sbtc.address.clone(),
    )?
    .total_supply
    .ok_or_else(|| Error::contract_err("sbtc total supply is private"))?;
    Ok(HandleResponse {
        messages: vec![
            snip20::mint_msg(
                env.message.sender.clone(),
                amount.into(),
                None,
                None,
                BLOCK_SIZE,
                config.sbtc.hash,
                config.sbtc.address,
            )?,
            finance_admin::CommonHandleMsg::SendMintReward {
                minter: env.message.sender.clone(),
                sbtc_mint_amount: amount.into(),
                sbtc_total_supply,
            }
            .to_cosmos_msg(
                config.finance_admin.hash,
                config.finance_admin.address,
                None,
            )?,
            log::HandleMsg::AddEvents {
                events: vec![(
                    env.message.sender,
                    log::Event::MintCompleted(log::event::MintCompletedData {
                        time: env.block.time,
                        address: bitcoin_address.to_string(),
                        amount: amount,
                        txid: txid.to_string(),
                    }),
                )],
            }
            .to_cosmos_msg(config.log.hash, config.log.address, None)?,
        ],
        log: vec![],
        data: None,
    })
}

fn try_release_incorrect_amount_btc<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    height: u32,
    tx: Binary,
    merkle_proof: bitcoin_spv::MerkleProofMsg,
    recipient_address: String,
    fee_per_vb: u64,
) -> Result<HandleResponse, Error> {
    let mintor = deps.api.canonical_address(&env.message.sender)?;
    let config = read_config(&deps.storage, &deps.api)?;

    //
    // Validate Tx Confirmed
    //
    if let bitcoin_spv::QueryAnswer::VerifyMerkleProof { success } =
        (bitcoin_spv::QueryMsg::VerifyMerkleProof {
            height,
            tx: tx.clone(),
            merkle_proof,
        })
        .query(
            &deps.querier,
            config.bitcoin_spv.hash.clone(),
            config.bitcoin_spv.address.clone(),
        )?
    {
        if success {
            Ok(())
        } else {
            Err(Error::contract_err("merkle proof verification failed"))
        }
    } else {
        Err(Error::contract_err(
            "unexpected query answer from bitcoin spv",
        ))
    }?;

    let tx: Transaction = deserialize::<Transaction>(tx.as_slice())?;
    let txid = tx.txid();
    let release_to = recipient_address;
    let recipient_address = Address::from_str(&release_to)?;

    //
    // Validate that the tx has correct output destination and incorrect value
    //
    let network = query_bitcoin_network(&deps.querier, config.bitcoin_spv)?;
    let priv_key = read_mint_key(&deps.storage, &mintor, network)?
        .ok_or_else(|| Error::contract_err("message sender does not have mint address"))?;
    let bitcoin_address = Address::p2wpkh(&priv_key.public_key(), priv_key.network)?;
    remove_mint_key(&mut deps.storage, &mintor);
    let vout = extract_vout(&tx.output, &bitcoin_address)?;
    let amount = tx.output[vout as usize].value;
    if config.btc_tx_values.contains(&amount) {
        return Err(Error::contract_err("sent value is correct"));
    }

    //
    //  Release Utxo
    //
    let tx = sign_transaction(
        vec![txin(OutPoint { txid, vout })],
        vec![priv_key],
        amount,
        fee_per_vb,
        recipient_address,
    )?;
    Ok(HandleResponse {
        messages: vec![log::HandleMsg::AddEvents {
            events: vec![(
                env.message.sender,
                log::Event::ReleaseIncorrectAmountBTC(log::event::ReleaseIncorrectAmountBTCData {
                    time: env.block.time,
                    amount: amount.into(),
                    release_from: bitcoin_address.to_string(),
                    release_to: release_to,
                    txid: tx.txid().to_string(),
                }),
            )],
        }
        .to_cosmos_msg(config.log.hash, config.log.address, None)?],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ReleaseIncorrectAmountBTC {
            tx: Binary::from(serialize(&tx)),
        })?),
    })
}

fn try_request_release_btc<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: u64,
    entropy: Binary,
) -> Result<HandleResponse, Error> {
    let config = read_config(&deps.storage, &deps.api)?;
    if !config.btc_tx_values.contains(&amount) {
        return Err(Error::contract_err("invalid tx amount"));
    }
    let requester = deps.api.canonical_address(&env.message.sender)?;

    let mut utxo_queue = UtxoQueue::from_storage(&mut deps.storage, amount);
    let utxo = utxo_queue
        .dequeue()?
        .ok_or_else(|| Error::contract_err("no utxo"))?;
    let mut rng = update_prng(
        &mut deps.storage,
        PREFIX_PRNG,
        &requester,
        entropy.as_slice(),
    )?;
    let request_key = gen_request_key(&requester, &utxo, &mut rng)?;
    write_release_request_utxo(&mut deps.storage, &request_key, amount, utxo)?;

    let sbtc_total_supply = snip20::token_info_query(
        &deps.querier,
        BLOCK_SIZE,
        config.sbtc.hash.clone(),
        config.sbtc.address.clone(),
    )?
    .total_supply
    .ok_or_else(|| Error::contract_err("sbtc total supply is private"))?;
    let res = HandleResponse {
        messages: vec![
            snip20::burn_from_msg(
                env.message.sender.clone(),
                amount.into(),
                None,
                None,
                BLOCK_SIZE,
                config.sbtc.hash,
                config.sbtc.address,
            )?,
            finance_admin::CommonHandleMsg::ReceiveReleaseFee {
                releaser: env.message.sender.clone(),
                sbtc_release_amount: amount.into(),
                sbtc_total_supply,
            }
            .to_cosmos_msg(
                config.finance_admin.hash,
                config.finance_admin.address,
                None,
            )?,
            log::HandleMsg::AddEvents {
                events: vec![(
                    env.message.sender,
                    log::Event::ReleaseStarted(log::event::ReleaseStartedData {
                        time: env.block.time,
                        request_key: request_key,
                        amount: amount,
                    }),
                )],
            }
            .to_cosmos_msg(config.log.hash, config.log.address, None)?,
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RequestReleaseBtc { request_key })?),
    };
    Ok(res)
}

fn try_claim_released_btc<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    tx_result_proof: sfps::TxResultProof,
    header_hash_index: u64,
    encryption_key: Binary,
    recipient_address: String,
    fee_per_vb: u64,
) -> Result<HandleResponse, Error> {
    let recipient_address = Address::from_str(&recipient_address)?;
    let config = read_config(&deps.storage, &deps.api)?;

    // Verify Merkle Proof
    let request_key = match sfps::verify_tx_result_proof(
        &deps.querier,
        config.sfps,
        tx_result_proof,
        header_hash_index,
        encryption_key,
    )? {
        HandleAnswer::RequestReleaseBtc { request_key } => request_key,
        _ => return Err(Error::contract_err("failed to deserialize decrypted text")),
    };
    let requested_utxo = read_release_request_utxo(&deps.storage, &request_key)?
        .ok_or_else(|| Error::contract_err("No release request"))?;

    let network = query_bitcoin_network(&deps.querier, config.bitcoin_spv)?;

    let tx = sign_transaction(
        vec![txin(requested_utxo.utxo.outpoint())],
        vec![requested_utxo.utxo.priv_key(network)?],
        requested_utxo.value,
        fee_per_vb,
        recipient_address,
    )?;

    let res = HandleResponse {
        messages: vec![log::HandleMsg::AddEvents {
            events: vec![(
                env.message.sender,
                log::Event::ReleaseCompleted(log::event::ReleaseCompletedData {
                    time: env.block.time,
                    request_key: request_key,
                    txid: tx.txid().to_string(),
                    fee_per_vb: fee_per_vb,
                }),
            )],
        }
        .to_cosmos_msg(config.log.hash, config.log.address, None)?],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ClaimReleasedBtc {
            tx: Binary::from(serialize(&tx)),
        })?),
    };
    Ok(res)
}

fn try_change_finance_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_finance_admin: ContractReference,
) -> Result<HandleResponse, Error> {
    let mut config = read_config(&deps.storage, &deps.api)?;
    if env.message.sender != config.finance_admin.address {
        return Err(Error::contract_err("not finance admin"));
    }
    config.finance_admin = new_finance_admin;
    write_config(&mut deps.storage, config, &deps.api)?;
    Ok(HandleResponse::default())
}

fn try_change_owner<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_owner: HumanAddr,
) -> Result<HandleResponse, Error> {
    let mut config = read_config(&deps.storage, &deps.api)?;
    if env.message.sender != config.owner {
        return Err(Error::contract_err("not owner"));
    }
    config.owner = new_owner;
    write_config(&mut deps.storage, config, &deps.api)?;
    Ok(HandleResponse::default())
}

fn try_set_suspension_switch<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    suspension_switch: SuspensionSwitch,
) -> Result<HandleResponse, Error> {
    let config = read_config(&deps.storage, &deps.api)?;
    if env.message.sender != config.owner {
        return Err(Error::contract_err("not owner"));
    }
    set_suspension_switch(&mut deps.storage, &suspension_switch)?;
    Ok(HandleResponse::default())
}

fn try_release_btc_by_owner<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    tx_value: u64,
    max_input_length: u64,
    recipient_address: String,
    fee_per_vb: u64,
) -> Result<HandleResponse, Error> {
    let recipient_address = Address::from_str(&recipient_address)?;
    let config = read_config(&deps.storage, &deps.api)?;
    if env.message.sender != config.owner {
        return Err(Error::contract_err("not owner"));
    }
    let network = query_bitcoin_network(&deps.querier, config.bitcoin_spv)?;
    let mut tx_inputs = Vec::with_capacity(max_input_length as usize);
    let mut priv_keys = Vec::with_capacity(max_input_length as usize);
    for _ in 0..max_input_length {
        let mut utxo_queue = UtxoQueue::from_storage(&mut deps.storage, tx_value);
        match utxo_queue.dequeue()? {
            Some(utxo) => {
                tx_inputs.push(txin(utxo.outpoint()));
                priv_keys.push(utxo.priv_key(network)?);
            }
            None => break,
        }
    }
    let tx = sign_transaction(
        tx_inputs,
        priv_keys,
        tx_value,
        fee_per_vb,
        recipient_address,
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ReleaseBtcByOwner {
            tx: Binary::from(serialize(&tx)),
        })?),
    })
}

// https://github.com/bitcoin/bips/blob/master/bip-0143.mediawiki#Native_P2WPKH
fn script_code(pub_key: &PublicKey) -> Script {
    Builder::new()
        .push_opcode(all::OP_DUP)
        .push_opcode(all::OP_HASH160)
        .push_slice(&pub_key.pubkey_hash())
        .push_opcode(all::OP_EQUALVERIFY)
        .push_opcode(all::OP_CHECKSIG)
        .into_script()
}

fn fee(recipient_address: &Address, txin_count: u64, fee_per_vb: u64) -> u64 {
    vsize(recipient_address, txin_count) * fee_per_vb
}

// ceil(weight / 4.0)
fn vsize(recipient_address: &Address, txin_count: u64) -> u64 {
    (weight(recipient_address, txin_count) + 3) / 4
}

// https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki
// https://github.com/bitcoin/bips/blob/master/bip-0144.mediawiki
// Return Max Weight of Transaction after signed.
// Weight of Signature in P2WPKH Witness can be 71 or 72.
// This function calculates Max Weight as the weight of signature to be 72.
fn weight(recipient_address: &Address, txin_count: u64) -> u64 {
    const INPUT_CONSTANT_WEIGHT: u64 = 160; // (Transaction Hash(32) + Output Index(4) + Sequence Number(4)) * 4
    const P2WPKH_SCRIPT_SIG_WEIGHT: u64 = 4; // (Script Sig Length VarInt(1) + Script Sig(0)) * 4
    const P2WPKH_WITNESS_WEIGHT: u64 = 108; // Witness Count VarInt(1) + Signature Length VarInt(1) + Signature (71 or 72) + Pubkey Length Varint (1) + pubkey(33)
    const TX_CONSTANT_WEIGHT: u64 = 34; // (Version(4) + Lock Time(4)) * 4 + Marker(1) + Flag(1)
    const OUTPUT_CONSTANT_WEIGHT: u64 = 32; // Amount(8) * 4
    const TXOUT_COUNT_WEIGHT: u64 = 4; // Tx Count VarInt(1) * 4

    let script_pubkey_length = recipient_address.script_pubkey().len() as u64;
    let input_weight =
        (INPUT_CONSTANT_WEIGHT + P2WPKH_SCRIPT_SIG_WEIGHT + P2WPKH_WITNESS_WEIGHT) * txin_count;
    let output_weight = OUTPUT_CONSTANT_WEIGHT
        + (VarInt(script_pubkey_length).len() as u64 + script_pubkey_length) * 4;
    let txin_count_weight = VarInt(txin_count).len() as u64 * 4;
    TX_CONSTANT_WEIGHT + txin_count_weight + input_weight + TXOUT_COUNT_WEIGHT + output_weight
}

fn txin(outpoint: OutPoint) -> TxIn {
    TxIn {
        previous_output: outpoint,
        sequence: 4294967293, // 0xFFFFFFFF - 2 sequence signals the transaction is considered to have opted in to allowing replacement of itself
        ..Default::default()
    }
}

fn sign_transaction(
    tx_inputs: Vec<TxIn>,
    priv_keys: Vec<PrivateKey>,
    tx_value: u64,
    fee_per_vb: u64,
    recipient_address: Address,
) -> Result<Transaction, Error> {
    let tx_inputs_size = tx_inputs.len() as u64;
    let fee = fee(&recipient_address, tx_inputs_size, fee_per_vb);
    let spendable_value = tx_value * tx_inputs_size;
    let mut tx = Transaction {
        version: 2,
        lock_time: 0,
        input: tx_inputs,
        output: vec![TxOut {
            value: spendable_value.saturating_sub(fee),
            script_pubkey: recipient_address.script_pubkey(),
        }],
    };

    let sighash_type = SigHashType::All; // SIGHASH_ALL
    let mut bip143hasher = SigHashCache::new(&mut tx);
    for i in 0..priv_keys.len() {
        let priv_key = priv_keys[i];
        let pub_key = priv_key.public_key();
        let sighash = bip143hasher.segwit_signature_hash(
            i,
            &script_code(&pub_key),
            tx_value,
            sighash_type,
        )?;
        let signature = sign(&Message::parse_slice(&sighash[..])?, &priv_key.key).0;
        let mut with_hashtype = signature.serialize_der().as_ref().to_vec();
        with_hashtype.push(sighash_type.as_u32() as u8);
        bip143hasher.witness_mut(i).unwrap().push(with_hashtype);
        bip143hasher
            .witness_mut(i)
            .unwrap()
            .push(pub_key.to_bytes().to_vec());
    }
    Ok(tx)
}
