#!/bin/bash
set -e

# Install wasm-bindgen with `cargo install wasm-bindgen-cli`.
# Pass --run option to run after build (uses python).
# Files in OutDir is everything needed to run the web page.

OutDir=target/wasm_package

HttpServerAddress=127.0.0.1
HttpServerPort=8000

if [ ! -e .git ]; then
	echo "Must be run from repository root"
	exit 1
fi

#
# Extract project name from Cargo.toml
#

ProjName="$(cargo metadata --no-deps --format-version 1 |
        sed -n 's/.*"name":"\([^"]*\)".*/\1/p')"

#
# Build
#

if [ ! -e target ] ; then
    mkdir target
fi

cargo build \
	--release --no-default-features \
	--target wasm32-unknown-unknown \

WasmFile="$(cargo metadata --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/wasm32-unknown-unknown/release/$ProjName.wasm"

if [ ! -e "$WasmFile" ]; then
	echo "Script is borken, it expects file to exist: $WasmFile"
	exit 1
fi

[ ! -e "$OutDir" ] || rm -r "$OutDir"

BINDGEN_EXEC_PATH="${CARGO_HOME:-~/.cargo}/bin/wasm-bindgen"

if [ ! -e "$BINDGEN_EXEC_PATH" ] ; then
    echo "Please install wasm-bindgen, cannot generate the wasm output without it"
    exit 1
fi

$BINDGEN_EXEC_PATH \
	--no-typescript \
	--out-dir "$OutDir" \
	--target web \
	"$WasmFile"

#
# Copy files
#

cp scripts/wasm_build.html "$OutDir/index.html"
cp -r assets "$OutDir/assets"

#
# Rename JS
#

Count=0
for _ in $OutDir/*.js; do
	((Count+=1))
done

if [ $Count -ne 1 ]; then
	echo "Script is broken, must be 1 JS file matching mask"
	exit 1
fi

mv $OutDir/*.js "$OutDir/main.js"

#
# Run
#

if [ "$1" = "--run" ]; then
	python3 -m http.server --bind $HttpServerAddress --directory "$OutDir" $HttpServerPort &
	Job=$!
    Browser=${BROWSER:-chromium}
	$Browser http://$HttpServerAddress:$HttpServerPort/index.html
	kill $Job
fi
