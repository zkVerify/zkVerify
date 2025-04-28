#!/usr/bin/env python3

import argparse
import yaml
import os
import shutil
import random
from collections import OrderedDict

def create_folders(project_root):
    # Create the project root if it doesn't exist
    os.makedirs(project_root, exist_ok=True)
    print(f"Created project root: {project_root}")

    os.makedirs(os.path.join(project_root, 'envs', 'relay'), exist_ok=True)
    os.makedirs(os.path.join(project_root, 'envs', 'para'), exist_ok=True)
    os.makedirs(os.path.join(project_root, 'resources'), exist_ok=True)
    os.makedirs(os.path.join(project_root, 'resources', 'secrets'), exist_ok=True)
    print(f"Created 'envs' and 'resources' folders in {project_root}")

def copy_resources(project_root, chain_spec, para_chain_spec):
    shutil.copy2(chain_spec, os.path.join(project_root, 'resources', 'raw-chainspec.json'))
    shutil.copy2(para_chain_spec, os.path.join(project_root, 'resources', 'raw-para-chainspec.json'))

def node(name, chain_spec, env_type, image, secret = None, node_key = None):
    n = OrderedDict([
        ('image', image),
        ('volumes', [
            f"{chain_spec}:/data/chain_spec.json"
        ]),
        ('env_file', [
            f"./envs/{env_type}/.env.{name}"
        ]),
        ])
    if secret is not None:
        n['volumes'].append(f"{secret}:/data/config/secret_phrase.dat")
    if node_key is not None:
        n['volumes'].append(f"{node_key}:/data/config//data/config/node_key.dat")
    return n
    
def verifier(name, chain_spec, image):
    return node(name, chain_spec, 'relay', image, 
                secret=f"./resources/secrets/{name}.dat:/data/config/secret_phrase.dat", 
                node_key=f"./resources/secrets/{name}_nodekey.dat:/data/config/node_key.dat")
    
def local_relay_node(name, chain_spec, image):
    n = node(name, chain_spec, 'relay', image)
    n['ports'] = ['9944:9944', '30333:30333']
    return n
    
def para_node(name, chain_spec, relay_chain_spec, image, secret = None, node_key = None):
    n = node(name, chain_spec, 'para', image, secret, node_key)
    n['volumes'].append(f"{relay_chain_spec}:/data/relay_chain_spec.json")    
    return n

def collator(name, chain_spec, relay_chain_spec, image):
    return para_node(name, chain_spec, relay_chain_spec, image, 
                     node_key = f"./resources/secrets/{name}_nodekey.dat:/data/config/node_key.dat")

def local_para_node(name, chain_spec, relay_chain_spec, image):
    node = para_node(name, chain_spec, relay_chain_spec, image)
    node['ports'] = ['8844:9944', '20333:30333']
    return node
    
def get_compose(image_relay,
                image_para,
                chain_spec,
                para_chain_spec):
    return OrderedDict([
        ('version', '3'),
        ('services', OrderedDict([
            ['local_node', local_relay_node('local_node', chain_spec, image_relay)],
            ['validator_1', verifier('validator_1', chain_spec, image_relay)],
            ['validator_2', verifier('validator_2', chain_spec, image_relay)],
            ['local_paranode', local_para_node('local_paranode', para_chain_spec, chain_spec, image_para)],
            ['collator_1', collator('collator_1', para_chain_spec, chain_spec, image_para)],
            ['collator_2', collator('collator_2', para_chain_spec, chain_spec, image_para)],
        ]))
        ])

def create_compose_file(project_root, image_relay='horizenlabs/zkv-relay:latest', image_para='paratest:latest', 
                        chain_spec='./resources/raw-chainspec.json', para_chain_spec='./resources/raw-para-chainspec.json'):
    # Construct the full path to the YAML file
    yaml_path = os.path.join(project_root, "compose.yaml")
    # Handling OrderedDict to YAML
    yaml.add_representer(OrderedDict, lambda dumper, data: dumper.represent_mapping('tag:yaml.org,2002:map', data.items()))
    # Handling tuple to YAML
    yaml.add_representer(tuple, lambda dumper, data: dumper.represent_sequence('tag:yaml.org,2002:seq', data))
    
    with open(yaml_path, 'w') as file:
        yaml.dump(get_compose(image_relay, image_para, chain_spec, para_chain_spec), file)

def crete_validator_env(path, name):
    with open(path, 'w') as file:
        file.write(f"""
# RUST_LOG=debug

# Node config
ZKV_CONF_NAME="{name}"


ZKV_CONF_BASE_PATH="/data/node"
ZKV_CONF_VALIDATOR="yes"
ZKV_CONF_CHAIN="/data/chain_spec.json"                   
""")

def crete_rpc_relay_env(path, name):
    with open(path, 'w') as file:
        file.write(f"""
# RUST_LOG=debug

# Node config
ZKV_CONF_NAME="{name}"

ZKV_CONF_BASE_PATH="/data/node"
ZKV_CONF_CHAIN="/data/chain_spec.json"

ZKV_CONF_RPC_EXTERNAL="yes"
ZKV_CONF_RPC_CORS="all"
ZKV_CONF_PRUNING="archive"
""")

def crete_collator_env(path, name, identity):
    identity = identity.upper()
    with open(path, 'w') as file:
        file.write(f"""
# RUST_LOG=debug

# Node config
ZKV_CONF_NAME="{name}"

ZKV_CONF_{identity}="yes"
ZKV_CONF_COLLATOR="yes"
ZKV_CONF_BASE_PATH="/data/node"

ZKV_CONF_CHAIN="/data/chain_spec.json"

RC_CONF_CHAIN="/data/relay_chain_spec.json"
""")

def crete_rpc_para_env(path, name):
    with open(path, 'w') as file:
        file.write(f"""
# RUST_LOG=debug

# Node config
ZKV_CONF_NAME="{name}"

ZKV_CONF_BASE_PATH="/data/node"
ZKV_CONF_RPC_EXTERNAL="yes"
ZKV_CONF_RPC_CORS="all"
ZKV_CONF_PRUNING="archive"

ZKV_CONF_CHAIN="/data/chain_spec.json"
RC_CONF_CHAIN="/data/relay_chain_spec.json"
""")
        
def create_envs(project):
    envs = os.path.join(project, 'envs')
    relay = os.path.join(envs, 'relay')
    para = os.path.join(envs, 'para')
    crete_validator_env(os.path.join(relay, '.env.validator_1'), 'Validator1')
    crete_validator_env(os.path.join(relay, '.env.validator_2'), 'Validator2')
    crete_rpc_relay_env(os.path.join(relay, '.env.local_node'), 'RpcRelay')
    crete_collator_env(os.path.join(para, '.env.collator_1'), 'Collator1', 'alice')
    crete_collator_env(os.path.join(para, '.env.collator_2'), 'Collator2', 'bob')
    crete_rpc_para_env(os.path.join(para, '.env.local_paranode'), 'RpcRelay')
    
def generate_secrets(project, keys = {}):
    secrets = os.path.join(project,'resources','secrets')
    for name in ['validator_1', 'validator_2', 'collator_1', 'collator_2']:
        key = f"{name}_nodekey.dat"
        with open(os.path.join(secrets, key), 'w') as file:
            file.write(f"{hex(random.getrandbits(256))[2:]:0>64}")
    for (name, k) in keys.items():
        key = f"{name}.dat"
        with open(os.path.join(secrets, key), 'w') as file:
            file.write(k)
    

def main():
    parser = argparse.ArgumentParser(description="Modify a YAML file in the project and create necessary folders.")
    parser.add_argument("project_root", nargs='?', default=os.getcwd(), 
                        help="The root directory of the project (default: current directory)")
    parser.add_argument("-r", "--relay", default="horizenlabs/zkv-relay:latest", help="The relay chain docker image")
    parser.add_argument("-p", "--para", default="paratest:latest", help="The parachain chain docker image")
    parser.add_argument("-c","--chain-spec", default="staging/raw-chainspec.json", help="The relay chain spec file path")
    parser.add_argument("-C","--para-chain-spec", default="staging/raw-para-chainspec.json", help="The para chain spec file path")
    parser.add_argument("--validator1_key", default="//Validator1", help="Validator1 private key")
    parser.add_argument("--validator2_key", default="//Validator2", help="Validator2 private key")
    
    args = parser.parse_args()
    
    print(f"Use Relay image {args.relay}.")
    print(f"Use Parachain image {args.para}.")
    print(f"Use chain-spec from in {args.chain_spec}.")
    print(f"Use para-chain-spec from in {args.para_chain_spec}.")
    create_folders(args.project_root)
    copy_resources(args.project_root, args.chain_spec, args.para_chain_spec)
    generate_secrets(args.project_root, keys = {'validator_1': args.validator1_key, 'validator_2': args.validator2_key})
    create_compose_file(args.project_root, args.relay, args.para, './resources/raw-chainspec.json', './resources/raw-para-chainspec.json')
    create_envs(args.project_root)
    print(f"Create compose.yaml in {args.project_root} and all environments files.")

if __name__ == "__main__":
    main()
