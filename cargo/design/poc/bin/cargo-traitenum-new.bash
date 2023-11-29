#!/bin/bash
###
# Creates a new workspace and members for a traitenum deployment.
# Two members are created:
#   - lib => Contains all of the traits to be exported
#   - derive => Contains the generated derive macro
#
# End-users will import the lib crate for the traits, and the derive crate to derive enums for those traits.
###
set -o errexit -o privileged -o pipefail -o nounset
shopt -s extglob

# These values would come from the command line as arguments
WORKSPACE_DIR="jango"
LIB_NAME="jango"
LIB_DIR="lib"
DERIVE_NAME="jango-derive"
DERIVE_DIR="derive"

SOURCE_PATH="$(realpath "$(dirname "${BASH_SOURCE[0]}")")" 
ASSETS_PATH="$(realpath "${SOURCE_PATH}"/../assets)"
TRAITENUM_LIB_PATH="$(realpath "${SOURCE_PATH}/../../../../lib")"
TRAITENUM_MACRO_PATH="$(realpath "${SOURCE_PATH}/../../../../macro")"

# Paths
LIB_RELPATH="${WORKSPACE_DIR}/${LIB_DIR}"
DERIVE_RELPATH="${WORKSPACE_DIR}/${DERIVE_DIR}"

# Assets
ASSET_WORKSPACE_MANIFEST_TEMPLATE="${ASSETS_PATH}/workspace_Cargo.toml.template"
ASSET_LIB_MANIFEST_TEMPLATE="${ASSETS_PATH}/lib_Cargo.toml.template"
ASSET_DERIVE_MANIFEST_TEMPLATE="${ASSETS_PATH}/derive_Cargo.toml.template"

main () {
    echo "[traitenum] Creating workspace ..."
    mk_workspace
    echo "[traitenum] Creating lib ..."
    mk_lib
    echo "[traitenum] Creating derive ..."
    mk_derive
    echo "[traitenum] Configuring lib ..."
    config_lib
    echo "[traitenum] Configuring derive ..."
    config_derive
    echo "[traitenum] Building ..."
    build_workspace
    echo "[traitenum] Testing ..."
    test_workspace
    echo
    echo "Your traitenum workspace is ready."
 }

mk_workspace () {
    cargo new -q --lib "${WORKSPACE_DIR}"
    rm -rf "${WORKSPACE_DIR}/src"
    cat "${ASSET_WORKSPACE_MANIFEST_TEMPLATE}" \
        | sed -e s"/%{LIB_DIR}%/${LIB_DIR}/g" -e s"/%{DERIVE_DIR}%/${DERIVE_DIR}/g" \
        > ${WORKSPACE_DIR}/Cargo.toml
}

mk_lib () {
    cargo new -q --lib --name "${LIB_NAME}" "${WORKSPACE_DIR}/${LIB_DIR}" 2>/dev/null
    cat "${ASSET_LIB_MANIFEST_TEMPLATE}" \
        | sed -e s"/%{LIB_NAME}%/${LIB_NAME}/g" \
        > ${LIB_RELPATH}/Cargo.toml
}

mk_derive () {
    cargo new -q --lib --name "${DERIVE_NAME}" "${WORKSPACE_DIR}/${DERIVE_DIR}" 1>/dev/null
    cat "${ASSET_DERIVE_MANIFEST_TEMPLATE}" \
        | sed -e s"/%{DERIVE_NAME}%/${DERIVE_NAME}/g" \
        > ${DERIVE_RELPATH}/Cargo.toml
    echo "" > ${DERIVE_RELPATH}/src/lib.rs
}

config_lib () {
    cargo -q add --manifest-path "${LIB_RELPATH}/Cargo.toml" --path "${TRAITENUM_MACRO_PATH}" 
}

config_derive () {
    cargo -q add --manifest-path "${DERIVE_RELPATH}/Cargo.toml" proc-macro2
    cargo -q add --manifest-path "${DERIVE_RELPATH}/Cargo.toml" --path "${TRAITENUM_LIB_PATH}" 
    cargo -q add --manifest-path "${DERIVE_RELPATH}/Cargo.toml" --path "${LIB_RELPATH}" 
}

build_workspace () {
    local pwd
    pwd="$PWD"
    cd "$WORKSPACE_DIR"
    cargo build
    cd "$pwd"
}

test_workspace () {
    local pwd
    pwd="$PWD"
    cd "$WORKSPACE_DIR"
    cargo test 
    cd "$pwd"
}

main
