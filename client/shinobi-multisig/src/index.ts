#! /usr/bin/env node
import { CAC } from 'cac';
import { dirname } from 'path';
import { Config, Convert, loadConfig } from './config';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';
import { keyInYN, question, setDefaultOptions } from 'readline-sync';
import { buildClient } from './buildClient';
import { MultisigClient } from 'sbtc-js/build/contracts/multisig/MultisigClient';

async function queryTxSequence(client: MultisigClient, id: number) {
    const queryResult = await client.transactionStatus(id);
    const decodedMsg = JSON.parse(
        Buffer.from(queryResult.transaction.msg, 'base64').toString()
    );
    const signedAddresses: string[] = [];
    const unsignedAddresses: string[] = [];
    queryResult.config.signers.forEach((address, index) => {
        if (queryResult.signed_by.includes(index)) {
            signedAddresses.push(address);
        } else {
            unsignedAddresses.push(address);
        }
    });
    const executed = queryResult.config.required <= signedAddresses.length;
    console.dir(
        {
            multisigTxID: id,
            destination: queryResult.transaction.contract_addr,
            msg: decodedMsg,
            executed,
            required: queryResult.config.required,
            signed: signedAddresses,
            unsigned: unsignedAddresses,
        },
        { depth: null }
    );
}

async function main() {
    setDefaultOptions({ keepWhitespace: true });

    const cli = new CAC();

    cli.option('--config-path <path>', 'File path of config json file.', {
        default:
            process.env[process.platform == 'win32' ? 'USERPROFILE' : 'HOME'] +
            '/.shinobi-multisig/config.json',
    });

    cli.command('init', 'Init cli config.').action((options) => {
        const configPath = options.configPath;
        if (existsSync(configPath)) {
            console.log('Cli config is already defined.');
            if (!keyInYN('Overwrite config?')) {
                return;
            }
            console.log();
        }
        console.log('Cli init. config will be saved to ' + configPath);
        console.log(
            'The config file will include mnemonic phrase of your signer account.'
        );
        console.log('Please handle the file with care.');
        if (!keyInYN('Are you sure?')) {
            return;
        }
        console.log('\n1/3 Address of Multisig Contract');
        const multisigAddress = question('Multisig address:');
        console.log('\n2/3 LCD api url (default: https://api.secretapi.io)');
        const lcdURL = question('LCD URL:', {
            defaultInput: 'https://api.secretapi.io',
        });
        console.log('\n3/3 Mnemonic phrase of your signer account');
        const mnemonic = question('Mnemonic:');
        const config: Config = {
            multisigAddress,
            lcdURL,
            mnemonic,
        };
        mkdirSync(dirname(configPath), { recursive: true, mode: '700' });
        writeFileSync(configPath, Convert.configToJson(config), {
            mode: '700',
        });
        console.log('\nConfig Saved.');
    });

    cli.command('show-config', 'Print cli config.').action((options) => {
        console.log(loadConfig(options.configPath));
    });

    cli.command('show-account', 'Print signer account.').action(
        async (options) => {
            const config = loadConfig(options.configPath);
            const client = await buildClient(config);
            console.log(await client.signingCosmWasmClient.getAccount());
        }
    );

    cli.command(
        'query-multisig-status',
        'Query multisig contract status (signers, required signs, transaction count).'
    ).action(async (options) => {
        const config = loadConfig(options.configPath);
        const client = await buildClient(config);
        console.log(await client.multisigStatus());
    });

    cli.command('query-tx <multisig tx id>', 'Query multisig tx.').action(
        async (id, options) => {
            const config = loadConfig(options.configPath);
            id = parseInt(id);
            const client = await buildClient(config);
            await queryTxSequence(client, id);
        }
    );

    cli.command(
        'submit-tx <contract address> <msg json file path>',
        'Submit new multisig tx. Multisig tx id will be returned.'
    ).action(async (contractAddress, msgJSONPath, options) => {
        const config = loadConfig(options.configPath);
        const client = await buildClient(config);
        const contractCodeHash =
            await client.signingCosmWasmClient.getCodeHashByContractAddr(
                contractAddress
            );
        const msg = JSON.parse(readFileSync(msgJSONPath, 'utf8'));
        console.group('Tx to be added');
        console.log('Destination: ' + contractAddress);
        console.log('Msg: ' + JSON.stringify(msg));
        console.groupEnd();
        if (!keyInYN('Are you sure to submit the tx to multisig?')) {
            console.log('Cancel');
            return;
        }
        const executeResult = await client.submitTransaction(
            msg,
            contractCodeHash,
            contractAddress,
            []
        );
        console.log();
        console.dir(
            {
                multisigTxID: executeResult.answer,
                destination: contractAddress,
                msg: msg,
            },
            { depth: null }
        );
    });

    cli.command(
        'sign-tx <multisig tx id>',
        'Sign a multisig tx. Once the required number of signers have signed the tx, the tx will be executed'
    ).action(async (id, options) => {
        const config = loadConfig(options.configPath);
        id = parseInt(id);
        const client = await buildClient(config);
        console.group('Tx to be signed');
        await queryTxSequence(client, id);
        console.groupEnd();
        if (!keyInYN('Are you sure to sign the tx?')) {
            console.log('Cancel');
            return;
        }
        await client.signTransaction(id);
        console.log();
        console.log('Sign Completed.');
    });

    cli.help();
    cli.parse(process.argv, { run: false });
    if (!cli.matchedCommand) {
        cli.outputHelp();
    }
    await cli.runMatchedCommand();
}

main().catch((e) => {
    if (e instanceof Error && (e.name === 'CACError' || e.name == 'CliError')) {
        console.error('Cli Error: ' + e.message);
    } else {
        console.error('Command Exited with Unhandled Error');
        console.error(e);
    }
});
