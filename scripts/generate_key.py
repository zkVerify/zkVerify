#!/bin/env python3

import argparse
import os
import subprocess
import json


def run_key_generator(key_generator_path, args, capture_stderr=False):
    result = subprocess.run([key_generator_path] + args, capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Key generator failed: {result.stderr}")
    return result.stderr if capture_stderr else result.stdout


def parse_key_output(output):
    lines = output.strip().split('\n')
    return {
        'secret_phrase': lines[0].split(':')[1].strip(),
        'ss58_public_key': lines[5].split(':')[1].strip()
    }


def generate_secret_phrase(base_key, name_component, index):
    if base_key is None:
        return None
    return f"{base_key}///{name_component}/{index}"


def generate_validator_keys(key_generator_path, num_validators, base_key=None, separate_account=False):
    validators = []
    secrets = []

    # Create secrets folder if it doesn't exist
    os.makedirs('secrets', exist_ok=True)

    for i in range(num_validators):
        secret_phrase = None
        if base_key is not None:
            secret_phrase = generate_secret_phrase(base_key, "Validator", i)
            sr25519_output = run_key_generator(key_generator_path,
                                               ['key', 'inspect', secret_phrase, '--scheme', 'sr25519'])
        else:
            sr25519_output = run_key_generator(key_generator_path, ['key', 'generate'])

        sr25519_data = parse_key_output(sr25519_output)
        if secret_phrase is None:
            secret_phrase = sr25519_data['secret_phrase']

        ed25519_output = run_key_generator(key_generator_path, ['key', 'inspect', secret_phrase, '--scheme', 'ed25519'])
        ed25519_data = parse_key_output(ed25519_output)

        if separate_account:
            if base_key is not None:
                secret_account_phrase = generate_secret_phrase(base_key, "ValidatorAccount", i)
            else:
                secret_account_phrase = parse_key_output(run_key_generator(key_generator_path, ['key', 'generate']))[
                    'secret_phrase']
        else:
            secret_account_phrase = secret_phrase

        if base_key is not None:
            account_sr25519_output = run_key_generator(key_generator_path,
                                                       ['key', 'inspect', secret_account_phrase, '--scheme', 'sr25519'])
        else:
            account_sr25519_output = run_key_generator(key_generator_path, ['key', 'generate'])
        account_sr25519_data = parse_key_output(account_sr25519_output)

        # Generate node key
        node_key_file = f'secrets/validator_node_key_{i}'
        node_key_output = run_key_generator(key_generator_path, ['key', 'generate-node-key'])
        peer_id = run_key_generator(key_generator_path, ['key', 'generate-node-key'], capture_stderr=True).strip()

        # Save the node key to the file
        with open(node_key_file, 'w') as f:
            f.write(node_key_output.strip())

        validators.append({
            'account_public_key': account_sr25519_data['ss58_public_key'],
            'sr25519_public_key': sr25519_data['ss58_public_key'],
            'ed25519_public_key': ed25519_data['ss58_public_key'],
            'peer_id': peer_id
        })

        secrets.append({
            'account': sanitize_secret_phrase(base_key, secret_account_phrase),
            'secret_phrase': sanitize_secret_phrase(base_key, secret_phrase),
            'node_key_file': node_key_file
        })

    return validators, secrets


def sanitize_secret_phrase(base_key, secret_phrase):
    if base_key is not None and secret_phrase.startswith(base_key + "/"):
        secret_phrase = secret_phrase[len(base_key) + 1:]
    return secret_phrase


def generate_sr25519_keys(key_generator_path, num_keys, role, base_key=None):
    keys = []
    secrets = []
    for i in range(num_keys):
        secret_phrase = None
        if base_key is not None:
            secret_phrase = generate_secret_phrase(base_key, role.replace("_", ""), i)
            output = run_key_generator(key_generator_path, ['key', 'inspect', secret_phrase, '--scheme', 'sr25519'])
        else:
            output = run_key_generator(key_generator_path, ['key', 'generate'])
        data = parse_key_output(output)
        if secret_phrase is None:
            secret_phrase = data['secret_phrase']
        keys.append({
            'sr25519_public_key': data['ss58_public_key']
        })

        secrets.append({
            'secret_phrase': sanitize_secret_phrase(base_key, secret_phrase)
        })
    return keys, secrets


def main():
    parser = argparse.ArgumentParser(description="Configure network parameters and generate keys")

    parser.add_argument("key_generator_path", type=str,
                        help="Path to the key generator program (mandatory)")
    parser.add_argument("-v", "--validators", type=int, default=10,
                        help="Number of validators (default: 10)")
    parser.add_argument("-n", "--nominators", type=int, default=10,
                        help="Number of nominators (default: 10)")
    parser.add_argument("-c", "--community_custodians", type=int, default=3,
                        help="Number of community custodians (default: 3)")
    parser.add_argument("-f", "--foundations", type=int, default=2,
                        help="Number of foundations (default: 2)")
    parser.add_argument("-cc", "--contributor_custodians", type=int, default=2,
                        help="Number of contributor custodians (default: 2)")
    parser.add_argument("-i", "--investors", type=int, default=2,
                        help="Number of investors (default: 2)")
    parser.add_argument("--disable-sudo", action="store_true",
                        help="Disable sudo account")
    parser.add_argument("--separate-account-validator", action="store_true",
                        help="Generate separate account for validators")
    parser.add_argument("--output", type=str, default="generated_keys.json",
                        help="Output file name for generated keys (default: generated_keys.json)")
    parser.add_argument("--secrets-folder", type=str, default="secrets",
                        help="Folder to store secret information (default: secrets)")
    parser.add_argument("--base-key", type=str, default=None,
                        help="Base key for generating derived keys (optional)")
    args = parser.parse_args()

    if not os.path.exists(args.key_generator_path):
        parser.error(f"The key generator path does not exist: {args.key_generator_path}")

    print(f"Key generator path: {args.key_generator_path}")
    print(f"Number of validators: {args.validators}")
    print(f"Number of nominators: {args.nominators}")
    print(f"Number of community custodians: {args.community_custodians}")
    print(f"Number of foundations: {args.foundations}")
    print(f"Number of contributor custodians: {args.contributor_custodians}")
    print(f"Number of investors: {args.investors}")
    print(f"Sudo account disabled: {args.disable_sudo}")
    print(f"Separate account validator: {args.separate_account_validator}")
    print(f"Output file: {args.output}")
    print(f"Secrets folder: {args.secrets_folder}")
    print(f"Base key: {'None' if args.base_key is None else args.base_key}")

    keys = {}
    secrets = {}

    # Create secrets folder if it doesn't exist
    os.makedirs(args.secrets_folder, exist_ok=True)

    keys['validators'], secrets['validators'] = generate_validator_keys(args.key_generator_path, args.validators,
                                                                        args.base_key, args.separate_account_validator)
    keys['nominators'], secrets['nominators'] = generate_sr25519_keys(args.key_generator_path, args.nominators,
                                                                      "Nominator", args.base_key)
    keys['community_custodians'], secrets['community_custodians'] = generate_sr25519_keys(args.key_generator_path,
                                                                                          args.community_custodians,
                                                                                          "CommunityCustodian",
                                                                                          args.base_key)
    keys['foundations'], secrets['foundations'] = generate_sr25519_keys(args.key_generator_path, args.foundations,
                                                                        "Foundation", args.base_key)
    keys['contributor_custodians'], secrets['contributor_custodians'] = generate_sr25519_keys(args.key_generator_path,
                                                                                              args.contributor_custodians,
                                                                                              "ContributorCustodian",
                                                                                              args.base_key)
    keys['investors'], secrets['investors'] = generate_sr25519_keys(args.key_generator_path, args.investors, "Investor",
                                                                    args.base_key)
    if not args.disable_sudo:
        keys['sudo'], secrets['sudo'] = generate_sr25519_keys(args.key_generator_path, 1, "Sudo", args.base_key)
        keys['sudo'] = keys['sudo'][0]  # We only need one sudo key
        secrets['sudo'] = secrets['sudo'][0]

    if args.base_key is not None:
        secrets['base_key'] = args.base_key

    with open(args.output, 'w') as f:
        json.dump(keys, f, indent=2)

    with open(os.path.join(args.secrets_folder, 'secrets.json'), 'w') as f:
        json.dump(secrets, f, indent=2)

    print(f"Generated keys for all roles and saved to {args.output}")
    print(f"Secret information saved to {os.path.join(args.secrets_folder, 'secrets.json')}")
    print(f"Node keys for validators saved in the '{args.secrets_folder}' folder")


if __name__ == "__main__":
    main()
