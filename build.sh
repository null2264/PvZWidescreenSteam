#!/bin/sh

# REF: https://jake-shadle.github.io/xwin

[ -d $PWD/xwin ] || (
	echo "Fetching SDK for cross compilation..."

	mkdir $PWD/xwin
	curl --fail -L https://github.com/Jake-Shadle/xwin/releases/download/0.6.6-rc.2/xwin-0.6.6-rc.2-x86_64-unknown-linux-musl.tar.gz | tar -xzv -C $PWD/xwin/ --strip-components=1 xwin-0.6.6-rc.2-x86_64-unknown-linux-musl/xwin || rm -r $PWD/xwin
	[ -d $PWD/xwin ] || exit 1

	chmod +x $PWD/xwin/xwin || exit 1
	$PWD/xwin/xwin --accept-license --arch x86 splat --output $PWD/xwin

	rm -rf $PWD/.xwin-cache/  # clean up
)

rustup target add i686-pc-windows-msvc

RUSTFLAGS="-Lnative=$PWD/xwin/crt/lib/x86 -Lnative=$PWD/xwin/sdk/lib/um/x86 -Lnative=$PWD/xwin/sdk/lib/ucrt/x86" \
CL_FLAGS="-Wno-unused-command-line-argument -fuse-ld=lld-link $PWD/xwin/crt/include $PWD/xwin/sdk/include/ucrt $PWD/xwin/sdk/include/um $PWD/xwin/sdk/include/shared" \
CFLAGS_i686_pc_windows_msvc="$CL_FLAGS" \
CXXFLAGS_i686_pc_windows_msvc="$CL_FLAGS" \
CARGO_TARGET_I686_PC_WINDOWS_MSVC_LINKER=lld-link \
	cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target i686-pc-windows-msvc --release
