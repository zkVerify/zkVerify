DEFAULT_BM_STEPS=50
DEFAULT_BM_REPEAT=20
DEFAULT_BM_HEAP_PAGES=4096

# The following line ensure we know the project root
PROJECT_ROOT=${PROJECT_ROOT:-$(git rev-parse --show-toplevel)}
WEIGTH_OUT_PATH=${WEIGTH_OUT_PATH:-""}
ZKV_NODE_EXE=${ZKV_NODE_EXE:-"${PROJECT_ROOT}/target/production/zkv-node"}
ZKV_RUNTIME=${ZKV_RUNTIME:-"${PROJECT_ROOT}/target/production/wbuild/zkv-runtime/zkv_runtime.compact.compressed.wasm"}
BM_STEPS=${BM_STEPS:-${DEFAULT_BM_STEPS}}
BM_REPEAT=${BM_REPEAT:-${DEFAULT_BM_REPEAT}}
BM_HEAP_PAGES=${BM_HEAP_PAGES:-${DEFAULT_BM_HEAP_PAGES}}
