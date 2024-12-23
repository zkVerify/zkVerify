# Create compose relay env without devel user (at least not the already registered ones)

These are notes about how to create a compose file and all need resources to run a network
composed of

- 2 Relay chain validator with **no develop account**
- 1 Relay rpc node
- 2 Parachain collator with develop account (Alice and Bob)
- 1 Parachain rpc node

In order to generate this network we should do the follow steps:

- Generate the relay chain and parachain docker images
- Generate validators credentials and secret files
- Generate relay chain spec and modify it
- Generate parachain spec files, genesis state and genesis wasm.
- Run python script to create compose file and network resources
- start the network

We assume now that we already built the two docker images

- `horizenlabs/zkv-relay` for relay chain
- `paratest` for relay chain

and we created a new empty folder `new_network` with a `staging` subfolder

```sh
> mkdir -p new_network/staging
> cd new_network
```

We will always use the docker image every time we need to run the node to build some
useful file.

## Generate validators credentials and secret files

For the two validators we'll use some derivatives from polkadot development account. We need

- private derivate path
  - `//Validator1`
  - `//Validator2`
- Their `sr25519` public key
- Their `ed25519` public key

To get the sr25519 use

```sh
docker run --rm -a stdout --entrypoint "" horizenlabs/zkv-relay zkv-relay key inspect "//Validator1" | grep "SS58 Address" | awk '{print $3}'
docker run --rm -a stdout --entrypoint "" horizenlabs/zkv-relay zkv-relay key inspect "//Validator1" --scheme "ed25519" | grep "SS58 Address" | awk '{print $3}'
docker run --rm -a stdout --entrypoint "" horizenlabs/zkv-relay zkv-relay key inspect "//Validator2" | grep "SS58 Address" | awk '{print $3}'
docker run --rm -a stdout --entrypoint "" horizenlabs/zkv-relay zkv-relay key inspect "//Validator2" --scheme "ed25519" | grep "SS58 Address" | awk '{print $3}'
```

| Keys           | sr25519                                            | ed25519                                            |
| -------------- | -------------------------------------------------- | -------------------------------------------------- |
| `//Validator1` | `5Hb48vxYpQZQHRbzLMak1AKhtuww6wNK1oMFyigysK8zvHyW` | `5CpBzkEqzW5RPjDqMMqJuTSkaNrdbo3WxYC8arVjysd96DCN` |
| `//Validator2` | `5G6win2P1ty9X6DYuvfSFkijHGVx8yceVp1uNFntENyu7H4j` | `5CkhLKX285NbSy4FioEBTwF7Q5X69wmr2d8z58tWr1V8sZK4` |

## Generate relay chain spec and modify it

Here we should generate the spec file and replace the validator public keys with the one that
we had computed in the previous chapter.

```sh
docker run --rm -a STDOUT \
    --entrypoint "" \
    horizenlabs/zkv-relay \
    zkv-relay build-spec \
        --chain local \
        --disable-default-bootnode \
        > staging/plain-chainspec.json
```

So `staging/plain-chainspec.json` contains the base chain spec for `local` chain: this chain use.
`Alice` and `Bob` as validators and we want to replace them by `Validator1` and `Validator2`:

Open the file with an editor and replace the `Alice` and `Bob` public keys with the new ones:
  
- sr25519 Alice `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY` -> sr25519 Validator1 `5Hb48vxYpQZQHRbzLMak1AKhtuww6wNK1oMFyigysK8zvHyW`
- ed25519 Alice `5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu` -> ed25519 Validator1 `5CpBzkEqzW5RPjDqMMqJuTSkaNrdbo3WxYC8arVjysd96DCN`
- sr25519 Alice `5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty` -> sr25519 Validator2 `5G6win2P1ty9X6DYuvfSFkijHGVx8yceVp1uNFntENyu7H4j`
- ed25519 Alice `5GoNkf6WdbxCFnPdAnYYQyCjAKPJgLNxXwPjwTh6DGg6gN3E` -> ed25519 Validator2 `5CkhLKX285NbSy4FioEBTwF7Q5X69wmr2d8z58tWr1V8sZK4`

Or use `sed`:

```sh
sed -i 's~5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY~5Hb48vxYpQZQHRbzLMak1AKhtuww6wNK1oMFyigysK8zvHyW~g' staging/plain-chainspec.json
sed -i 's~5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu~5CpBzkEqzW5RPjDqMMqJuTSkaNrdbo3WxYC8arVjysd96DCN~g' staging/plain-chainspec.json
sed -i 's~5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty~5G6win2P1ty9X6DYuvfSFkijHGVx8yceVp1uNFntENyu7H4j~g' staging/plain-chainspec.json
sed -i 's~5GoNkf6WdbxCFnPdAnYYQyCjAKPJgLNxXwPjwTh6DGg6gN3E~5CkhLKX285NbSy4FioEBTwF7Q5X69wmr2d8z58tWr1V8sZK4~g' staging/plain-chainspec.json
```

Now we can create the raw spec file:

```sh
docker run --rm -a STDOUT \
        -v "./staging/plain-chainspec.json:/plain-chainspec.json" \
        --entrypoint "" \
        horizenlabs/zkv-relay \
        zkv-relay build-spec \
            --chain /plain-chainspec.json \
            --raw \
            --disable-default-bootnode > staging/raw-chainspec.json
```

## Generate parachain spec files, genesis state and genesis wasm

```sh
docker run --rm -a STDOUT \
    --entrypoint "" \
    paratest \
    paratest-node build-spec \
        --chain local \
        --disable-default-bootnode \
        > staging/plain-parachain-chainspec.json
```

```sh
docker run --rm -a STDOUT \
    -v "./staging/plain-parachain-chainspec.json:/plain-chainspec.json" \
    --entrypoint "" \
    paratest \
    paratest-node build-spec \
        --chain /plain-chainspec.json \
        --disable-default-bootnode \
        --raw \
        > staging/raw-para-chainspec.json
```

```sh
docker run --rm -a STDOUT \
    -v "./staging/raw-para-chainspec.json:/raw-chainspec.json" \
    --entrypoint "" \
    paratest \
    paratest-node export-genesis-state \
        --chain /raw-chainspec.json \
        > staging/para-genesis
```

```sh
docker run --rm -a STDOUT \
    -v "./staging/raw-para-chainspec.json:/raw-chainspec.json" \
    --entrypoint "" \
    paratest \
    paratest-node export-genesis-wasm \
        --chain /raw-chainspec.json \
        > staging/para-wasm
```

## Run python script to create compose file and network resources

Now we assume that `create_relay_compose.py` is in your path (otherwise use a complete
path).

```sh
mdamico@miklap:~/devel/zkVerify/new_network$ create_relay_compose.py --help
usage: create_relay_compose.py [-h] [-r RELAY] [-p PARA] [-c CHAIN_SPEC] [-C PARA_CHAIN_SPEC] [--validator1_key VALIDATOR1_KEY] [--validator2_key VALIDATOR2_KEY] [project_root]

Modify a YAML file in the project and create necessary folders.

positional arguments:
  project_root          The root directory of the project (default: current directory)

options:
  -h, --help            show this help message and exit
  -r RELAY, --relay RELAY
                        The relay chain docker image
  -p PARA, --para PARA  The parachain chain docker image
  -c CHAIN_SPEC, --chain-spec CHAIN_SPEC
                        The relay chain spec file path
  -C PARA_CHAIN_SPEC, --para-chain-spec PARA_CHAIN_SPEC
                        The para chain spec file path
  --validator1_key VALIDATOR1_KEY
                        Validator1 private key
  --validator2_key VALIDATOR2_KEY
                        Validator2 private key

```

If you used the value from this tutorial just run the script with the default value should
work. If you changed something you should change the flags values accordantly.

```sh
mdamico@miklap:~/devel/zkVerify/new_network$ create_relay_compose.py 
Use Relay image horizenlabs/zkv-relay:latest.
Use Parachain image paratest:latest.
Use chain-spec from in staging/raw-chainspec.json.
Use para-chain-spec from in staging/raw-para-chainspec.json.
Created project root: /home/mdamico/devel/zkVerify/new_network
Created 'envs' and 'resources' folders in /home/mdamico/devel/zkVerify/new_network
Create compose.yaml in /home/mdamico/devel/zkVerify/new_network and all environments files.
```

Now start:

```sh
> docker compose up
```

It exposes `9944` and `8844` on local host for both relay chain and parachain. In order to operate
on this relay chain network you need the 2 validators key on your wallet or on `polkadot.js` directly.

### Add an account on polkadot js

1. On _Settings_ -> _account options_ -> **In-browser account creation** : click and allow
2. On _Accounts_ -> _My accounts_ -> **+ Account**
3. Switch _Mnemonic_ into _Development_ (to use polkadot development seed phrase)
4. On _Advanced creation option_ -> **secret derivation path** -> `Validator1`
5. Set your name and password.
