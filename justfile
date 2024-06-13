set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default: build-full

alias b := build-base
#alias bp := build-plugin
alias bf := build-full

[doc('Build the base binary')]
@build-base:
    cargo build --release
    mv target/release/wootili-view.exe wootili-view.exe

[doc('Build the base binary and all the plugins')]
@build-full: build-base
    cd plugins/update_check; cargo build --release; mv target/release/update_check.dll ../../update_check.dll