# zkVerify zombienet tests

## TLDR

### Requirements for a local run

- node
- npm
- yarn

### Brief instructions for executing the tests

- Compile zkVerify relay chain (debug or release mode).
- Run from within the `zombienet-tests` directory:
- - `yarn install`
- - `yarn test`

This will add the profile target folder to the path and will download the zombienet binary in the same directory.

It will then execute all the end-to-end test (all the `.zndsl` file listed in `test_runner.sh`) for the relay chain.

---

## Description

This folder contains the `zkVerify` tests written for the Polkadot zombienet testing framework.

Each zombienet test requires the following files:

- A network configuration specification file, in `.toml` format; these are stored in `zombienet-tests/network_defs`
- The test description, written in zombienet DSL and stored in a `.zndsl` file. If the test description contains
  a comment line of the form `# SKIP <node>` placed after the network description, this test will be skipped for 
  this kind of node.
- (optional) One or more Javascript / Typescript file containing complex tests whose logic cannot be expressed 
  in zombienet DSL language; these are stored in the `zombienet-tests/js_scripts` directory.

## Running the test

### Prerequisites

All tests are executed by running actual `zkVerify` nodes, so an instance of the `zkv-relay` executable must be present on the target system.
This can be obtained by compiling this repository, as the test runner script looks for the compiled binary in the `target/debug/` or `target/release/` directories. Optionally, it is possible to download the pre-compiled binary from our official `zkVerify` repository, and copy that in the `zombienet-tests/bin` directory.

The test runner script also automatically downloads the `zombienet` executable from the official GitHub repo: <https://github.com/paritytech/zombienet/releases> and places that in the `zombienet-tests/bin` as well.

Finally, for local execution, the following packages must be present in the system:

- node
- npm
- yarn

### Execute the test suite entirely

Run from within the `zombienet-tests` directory:

- `yarn install`
- `yarn test`

The last command invokes the script `test_runner.sh`. If you want to use the binary compiled in
debug flavor you can use `--debug` switch. The following line executes all the tests running the node binary
from `target/debug` folder.

```sh
yarn test --debug
```

### Execute a single test

You can run a list of tests by simply specifying the names while running the tests:

```sh
yarn test 0005-proofPath_rpc.zndsl 0006-proof_should_pay.zndsl
```

Execute exactly `0005-proofPath_rpc.zndsl`, `0006-proof_should_pay.zndsl` and nothing else.

Anyway you can call directly _zombienet_ binary to execute a test: The following instructions are for the
local execution environment only, without taking into account Kubernetes or Podman - supported by zombienet, 
but not mandatory.

Test are launched with the `zombienet test` command, followed by the `.zndsl` file, for instance:

```sh
bin/zombienet-linux-x64 -p native test 0003-transaction.zndsl
```

Node instances are automatically shut down at the end of the test execution.

It is possible to get additional logging information by setting the `DEBUG` environmental variable appropriately:

```sh
DEBUG=zombie* bin/zombienet-linux-x64 -p native test 0003-transaction.zndsl
```

### Spawn a local network

Optionally, it is possible to use zombienet to spawn one of the test network configuration, without actually running a test. To this end, we use the `zombienet spawn` command:

```sh
cd zombienet/0003-transaction
bin/zombienet-linux-x64 -p native spawn network_defs/two_nodes.toml
```

Nodes keep running indefinitely, and must be stopped with CTRL-C.

## Test development

The official documentation is available at the following link <https://paritytech.github.io/zombienet/>
However, it is quite basic with regard to actual test development.

Here we collect some additional information gathered from testing the framework and analyzing its source code.

## Test format

As described in the introduction, basic tests can be created by writing a network definition spec file and test description file, written in a natural language style.

A brief definition of the DSL employed for writing the test is provided here <https://paritytech.github.io/zombienet/cli/test-dsl-definition-spec.html>

It provides basic commands for controlling the nodes, and a set of hard coded assertions which can be triggered according to:

- On chain storage
- Metrics
- Histograms
- Logs
- System events
- Tracing
- Custom api calls (through polkadot.js)
- Commands

We managed to find a few examples on tests written in this DSL at the following links:

- <https://github.com/paritytech/zombienet/tree/main/tests>
- <https://github.com/paritytech/zombienet/tree/main/tests/smoke>
- <https://github.com/paritytech/polkadot-sdk/tree/master/substrate/zombienet>

Other than that, we could not find the official specifications for the DLS grammar, but the list of accepted assertions and commands is available here:

- <https://github.com/paritytech/zombienet/blob/main/javascript/packages/orchestrator/src/test-runner>

while the list of the accepted tokens is included in this file:

- <https://github.com/paritytech/zombienet/blob/main/crates/parser/src/zombienet.pest>

The actual parser <https://github.com/paritytech/zombienet/blob/main/crates/parser/src/lib.rs> shows also a hint of the undocumented `run` command, which allows the execution of an arbitrary shell script, eventually triggering an assertion depending on the script return value.

## Writing complex tests

The DSL alone does not allow to create complex testing scenarios, as the selection of commands is quite limited. Luckily, it is possible to interact with polkadot.js APIs to execute specific actions on a node (or a set of nodes) - for instance, reading the list of validators, getting the current block height, or execute a transaction.

Hence, complex tests can be written as js/ts scripts and executed in zombienet DSL through the commands `js-script` or `ts-script`; eventually, an assertion can be triggered depending on the return value of this script (a single u64 value).

The big downside is that there is no documentation for this, and the provided examples are quite dull.

A good starting point are the example 0003 and 0004 in this repository. They provide two examples on how to perform a successful / unsuccessful transaction between two nodes.

In general, reading the polkadot.js documentation provides quite a few important hints on how to interact with the nodes. Also, a comprehensive list of the functions exposed by the polkadot.js API is available here:

- <https://polkadot.js.org/docs/substrate/>
- <https://polkadot.js.org/docs/polkadot/>    <- Not sure it can be used
- <https://polkadot.js.org/docs/kusama/>      <- Not sure it can be used

## Various

It is possible to use polkadot.js dashboard to test small js snippets of code. To access this, you can use zombienet to spawn a simple network configuration, connect to the http address returned by the testing env, and navigate to the "Developer->Javascript" pane.

There are quite a few small hints on how to write tests in the examples of this folders. I left a few more comments in each test src. As an example, test 0002 shows how to pass an argument to the js script from the zombienet DSL.
