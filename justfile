set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default: build-full

alias b := build-base
#alias bp := build-plugin
alias bf := build-full
alias c := clean

[doc('Builds/Rebuilds the base binary')]
@build-base: cleanbin
    cargo build --release
    mv target/release/wootili-view.exe wootili-view.exe

[doc('Builds/Rebuild the base binary and all the plugins')]
@build-full: build-base
    cd plugins/update_check; cargo build --release; mv target/release/update_check.dll ../../update_check.dll

[doc('Clean the project')]
@clean: cleanbin
    cargo clean

[doc('Cleans final bins')]
@cleanbin:
    if ( Test-Path -path wootili-view.exe ) { rm wootili-view.exe }
    if ( Test-Path -path update_check.dll ) { rm update_check.dll }

