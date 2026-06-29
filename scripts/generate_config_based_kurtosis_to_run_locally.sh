#!/usr/bin/env bash
# scripts/generate_config_based_kurtosis_to_run_locally.sh
#
# Downloads the aggkit-prover config from the kurtosis enclave 'op' and
# replaces internal kurtosis hostnames with the public ports exposed by
# the enclave. Output is written to ./tmp/aggkit-prover-config.toml.
#
# Usage:
#   ./scripts/generate_config_based_kurtosis_to_run_locally.sh
#   ./scripts/generate_config_based_kurtosis_to_run_locally.sh my-enclave  # custom enclave name

set -euo pipefail

# Associative arrays require Bash 4.2+. macOS ships Bash 3.2 — install a newer
# version with: brew install bash
if (( BASH_VERSINFO[0] < 4 || (BASH_VERSINFO[0] == 4 && BASH_VERSINFO[1] < 2) )); then
    echo "ERROR: Bash 4.2+ required (found ${BASH_VERSION})."
    echo "       On macOS: brew install bash"
    exit 1
fi

# Portable sed -i: macOS requires an explicit backup extension (we use '').
sedi() {
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "$@"
    else
        sed -i "$@"
    fi
}

# Portable realpath: resolve to an absolute path without GNU coreutils.
portable_realpath() {
    local path="$1"
    if command -v realpath &>/dev/null; then
        realpath "$path"
    else
        (cd "$path" && pwd)
    fi
}

ENCLAVE="${1:-op}"
CONFIG_ARTIFACT="aggkit-prover-config-001"
EVM_GENESIS_ARTIFACT="evm-sketch-genesis-conf-artifact.json"
TMP_DIR="./tmp"
DL_DIR="$TMP_DIR/_kurtosis_downloads"
OUTPUT_CONFIG="$TMP_DIR/aggkit-prover-config.toml"
INSPECT_OUT="$TMP_DIR/_enclave_inspect.txt"

# ── Setup ─────────────────────────────────────────────────────────────────────

mkdir -p "$TMP_DIR"
rm -rf "$DL_DIR" && mkdir -p "$DL_DIR"

# ── Inspect enclave ───────────────────────────────────────────────────────────

echo "==> Inspecting kurtosis enclave '$ENCLAVE'..."
_inspect_err="$TMP_DIR/_enclave_inspect_err.txt"
if ! kurtosis enclave inspect "$ENCLAVE" > "$INSPECT_OUT" 2>"$_inspect_err"; then
    echo "ERROR: kurtosis enclave inspect '$ENCLAVE' failed."
    cat "$_inspect_err"
    exit 1
fi

# ── Download config artifact ──────────────────────────────────────────────────

echo "==> Downloading aggkit-prover config from enclave '$ENCLAVE'..."
kurtosis files download "$ENCLAVE" "$CONFIG_ARTIFACT" "$DL_DIR/prover-config"
cp "$DL_DIR/prover-config/config.toml" "$OUTPUT_CONFIG"

# ── Download evm-sketch-genesis ───────────────────────────────────────────────

echo "==> Downloading evm-sketch-genesis from enclave '$ENCLAVE'..."
kurtosis files download "$ENCLAVE" "$EVM_GENESIS_ARTIFACT" "$DL_DIR/evm-sketch-genesis"
EVM_GENESIS_SRC=$(find "$DL_DIR/evm-sketch-genesis" -maxdepth 2 -name "*.json" | head -1)
if [[ -z "$EVM_GENESIS_SRC" ]]; then
    echo "ERROR: no JSON file found in artifact '$EVM_GENESIS_ARTIFACT'"
    exit 1
fi
LOCAL_EVM_GENESIS="$(portable_realpath "$TMP_DIR")/evm-sketch-genesis.json"
cp "$EVM_GENESIS_SRC" "$LOCAL_EVM_GENESIS"
echo "    saved to: $LOCAL_EVM_GENESIS"

# ── Build port map from kurtosis enclave inspect ──────────────────────────────
# Builds an associative array: PORT_MAP[service_name:internal_port] = public_url

echo "==> Building kurtosis port map for enclave '$ENCLAVE'..."
declare -A PORT_MAP
current_service=""

while IFS= read -r line; do
    # New service block: line starts with a 12-char hex UUID
    if [[ "$line" =~ ^[a-f0-9]{12}[[:space:]]+([a-zA-Z0-9_-]+) ]]; then
        current_service="${BASH_REMATCH[1]}"
    fi

    # Port with URL scheme: "NNNN/tcp -> scheme://127.0.0.1:PPPP"
    if [[ "$line" =~ ([0-9]+)/tcp[[:space:]]*-\>[[:space:]]*(https?|grpc|ws|wss)://127\.0\.0\.1:([0-9]+) ]]; then
        PORT_MAP["${current_service}:${BASH_REMATCH[1]}"]="${BASH_REMATCH[2]}://127.0.0.1:${BASH_REMATCH[3]}"
    # Port without URL scheme: "NNNN/tcp -> 127.0.0.1:PPPP"
    elif [[ "$line" =~ ([0-9]+)/tcp[[:space:]]*-\>[[:space:]]*127\.0\.0\.1:([0-9]+) ]]; then
        PORT_MAP["${current_service}:${BASH_REMATCH[1]}"]="http://127.0.0.1:${BASH_REMATCH[2]}"
    fi
done < "$INSPECT_OUT"

# ── Replace kurtosis internal URLs with public ports ─────────────────────────
# Finds every URL in the config matching a kurtosis internal hostname pattern
# (non-IP hostname) and replaces it with the public port from the enclave.

echo "==> Replacing kurtosis internal URLs with public ports..."
while IFS= read -r internal_url; do
    # Match scheme://kurtosis-hostname:port  (hostname starts with a letter, not an IP)
    if [[ "$internal_url" =~ ^(https?|grpc)://([a-zA-Z][a-zA-Z0-9_-]+):([0-9]+)$ ]]; then
        orig_scheme="${BASH_REMATCH[1]}"
        hostname="${BASH_REMATCH[2]}"
        internal_port="${BASH_REMATCH[3]}"
        key="${hostname}:${internal_port}"

        if [[ -v PORT_MAP["$key"] ]]; then
            # Preserve the scheme from the original config URL (e.g. http vs grpc).
            # Tonic gRPC clients accept http:// for plaintext connections.
            public_host_port="${PORT_MAP[$key]#*://}"
            replacement="${orig_scheme}://${public_host_port}"
            echo "    $internal_url  ->  $replacement"
            sedi "s|${internal_url}|${replacement}|g" "$OUTPUT_CONFIG"
        else
            echo "    WARNING: no public port found for '$internal_url' (key: $key)"
        fi
    fi
done < <(grep -oE '(https?|grpc)://[a-zA-Z][a-zA-Z0-9_-]+:[0-9]+' "$OUTPUT_CONFIG" | sort -u)

# ── Replace evm-sketch-genesis container path with local file ─────────────────

echo "==> Replacing evm-sketch-genesis path..."
sedi "s|/etc/aggkit-prover/evm-sketch-genesis.json|${LOCAL_EVM_GENESIS}|g" "$OUTPUT_CONFIG"

# ── Replace grpc-endpoint with the kurtosis public port for aggkit-prover-001 ─
# grpc-endpoint is the bind address of this service. Setting it to the same
# port kurtosis exposes means other services (agglayer, aggkit) can connect
# to the local process without reconfiguration.

PROVER_GRPC_PUBLIC="${PORT_MAP["aggkit-prover-001:4446"]:-}"
if [[ -n "$PROVER_GRPC_PUBLIC" ]]; then
    # Extract just the port number from e.g. "grpc://127.0.0.1:32800"
    PROVER_GRPC_PORT="${PROVER_GRPC_PUBLIC##*:}"
    echo "==> Setting grpc-endpoint to kurtosis public port: 127.0.0.1:$PROVER_GRPC_PORT"
    sedi "s|grpc-endpoint = \"[^\"]*\"|grpc-endpoint = \"127.0.0.1:${PROVER_GRPC_PORT}\"|" "$OUTPUT_CONFIG"
else
    echo "    WARNING: could not find public port for aggkit-prover-001:4446, keeping grpc-endpoint as-is"
fi

# ── Replace 0.0.0.0 listen addresses for local use ───────────────────────────

sedi 's|0\.0\.0\.0:|127.0.0.1:|g' "$OUTPUT_CONFIG"

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "Config written to: $OUTPUT_CONFIG"
echo ""
echo "To run:"
echo "  cargo run -p aggkit-prover -- run --config $OUTPUT_CONFIG"
