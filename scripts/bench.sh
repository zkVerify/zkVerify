#!/bin/bash

ECHO_CMD="${ECHO_CMD:-false}"
[ "${ECHO_CMD}" = "true" ] && set -x

PROJECT_ROOT=${PROJECT_ROOT:-$(git rev-parse --show-toplevel)}
SOURCE_ROOT=${SOURCE_ROOT:-${PROJECT_ROOT}}

. "${SOURCE_ROOT}/scripts/bench_cfg.sh"

DEFAULT_LOCAL_WEIGHT_TEMPLATE="${PROJECT_ROOT}/relay-node/benchmarks/zkv-pallets-weight-template.hbs"
DEFAULT_SKIP_BUILD="false"
DEFAULT_CODE_HEADER="${PROJECT_ROOT}/HEADER-APACHE2"

WEIGTH_TEMPLATE=${WEIGTH_TEMPLATE:-"${DEFAULT_LOCAL_WEIGHT_TEMPLATE}"}
SKIP_BUILD=${SKIP_BUILD:-"${DEFAULT_SKIP_BUILD}"}
CODE_HEADER=${CODE_HEADER:-"${DEFAULT_CODE_HEADER}"}

PALLET=$1

function usage {
    local message=${1:-""};

    echo "$0 <pallet> : get pallet crate name, execute benchamark and save the weigth file.
    Environment:
    WEIGTH_TEMPLATE : the template file path to use for rendering [${DEFAULT_LOCAL_WEIGHT_TEMPLATE}].
    WEIGTH_OUT_PATH : the path of the rendered weight file. If empty it will use <pallet_path>/src/weight.rs.
    BM_STEPS        : benchmark steps [${DEFAULT_BM_STEPS}].
    BM_REPEAT       : benchmark repeat [${DEFAULT_BM_REPEAT}].
    BM_HEAP_PAGES   : benchmark heap pages [${DEFAULT_BM_HEAP_PAGES}].
    BASE_PATH_ARG   : file path to use for disk I/O benchmarks, default CWD.
    CODE_HEADER     : the path for the header file to prepend to the template render [${DEFAULT_CODE_HEADER}].
    PROJECT_ROOT    : the root of the project [the root of git project].
    SOURCE_ROOT     : the root of the source [the root of git project].
    SKIP_BUILD      : skip the build step if true [${DEFAULT_SKIP_BUILD}].
    ZKV_NODE_EXE    : the path to the zkv-relay executable [target/production/zkv-relay in project root]
    ZKV_RUNTIME     : the path to the zkv-runtime wasm [target/production/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm in project root]
    "
    if [ -n "${message}" ]; 
    then
        echo "ERROR: $message"
    fi
    exit 1
}

check_cargo() {
    if ! cargo --list | grep -q -P "^\s+workspaces$" ; 
    then 
        usage "You need cargo-workspaces installed -> cargo install cargo-workspaces"
    fi
}

if [ -z "${PALLET}" ] ;
then
    usage
fi

if [ -z "${WEIGTH_OUT_PATH}" ];
then
    check_cargo

    if ! cargo workspaces list -l -a | grep -q -w "${PALLET}" ;
    then 
        usage "Pallet '${PALLET}' not found"
    fi

    CRATE_PATH=$(cargo workspaces list -l -a | grep -w  "${PALLET}" | awk '{print $3 }')
    
    WEIGTH_OUT_PATH="${CRATE_PATH}/src/weight.rs"
fi

echo "------------------------------------------------------------------
Use:
SKIP_BUILD=${SKIP_BUILD}
ZKV_NODE_EXE=${ZKV_NODE_EXE}
PALLET=${PALLET}
WEIGTH_OUT_PATH=${WEIGTH_OUT_PATH}
WEIGTH_TEMPLATE=${WEIGTH_TEMPLATE}
BM_STEPS=${BM_STEPS}
BM_REPEAT=${BM_REPEAT}
BM_HEAP_PAGES=${BM_HEAP_PAGES}
BASE_PATH_ARG=${BASE_PATH_ARG}
------------------------------------------------------------------"

if [ "${SKIP_BUILD}" = "false" ]; 
then
    check_cargo

    cd "${PROJECT_ROOT}" &&
    cargo build \
        --profile production \
        --locked \
        --features=runtime-benchmarks \
        --bin zkv-relay
    FAILED=$?
    cd - || exit 1
    if [ "${FAILED}" -ne 0 ]; then
        exit 1
    fi
fi

${ZKV_NODE_EXE} \
    benchmark pallet \
    --runtime "${ZKV_RUNTIME}" \
    --genesis-builder=runtime \
    --pallet "${PALLET}" \
    --extrinsic "*" \
    --steps "${BM_STEPS}" \
    --repeat "${BM_REPEAT}" \
    --heap-pages="${BM_HEAP_PAGES}" \
    --header "${CODE_HEADER}" \
    --output "${WEIGTH_OUT_PATH}" \
    --template "${WEIGTH_TEMPLATE}" \
    ${BASE_PATH_ARG}
