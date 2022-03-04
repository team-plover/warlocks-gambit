#!/bin/bash
set -e

# Install wasm-bindgen with `cargo install wasm-bindgen-cli`.
# Pass --run option to run after build (uses python).
# Files in OutDir is everything needed to run the web page.

OutDir=target/wasm_package
HttpServerAddress=0.0.0.0
HttpServerPort=8000

if [ ! -e .git ]; then
	echo "Must be run from repository root"
	exit 1
fi

#
# Extract project name from Cargo.toml
#

ProjName=`cargo pkgid`
ProjName=${ProjName##*#}
ProjName=${ProjName%:*}

#
# Build
#

cargo build \
	--release --no-default-features \
	--target wasm32-unknown-unknown \

WasmFile=target/wasm32-unknown-unknown/release/$ProjName.wasm

if [ ! -e "$WasmFile" ]; then
	echo "Script is borken, it expects file to exist: $WasmFile"
	exit 1
fi

[ ! -e "$OutDir" ] || rm -r "$OutDir"

$HOME/.cargo/bin/wasm-bindgen \
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
	chromium $HttpServerAddress:$HttpServerPort/index.html
	kill $Job
fi
