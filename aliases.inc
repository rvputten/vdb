function run_cargo {
    local a="-------------------------------------------------------------------------------"
    echo "$a$a$a$a$a" | cut -b-${COLUMNS:-79}
    cargo $*
}
alias d="cargo doc --open --document-private-items"
alias b="run_cargo build"
alias br="run_cargo build --release"
alias r="run_cargo run"
alias rr="run_cargo run --release"
alias t="run_cargo test"
alias gp="run_cargo test && git commit . -m inc && git push"

function d {
    exe=$(basename $0)
    rust-gdb target/debug/$exe
}

function v {
    local v=vim
    [[ -e /usr/bin/gvim ]] && v=gvim
    $v *md src/* *.toml run.sh
}

function killrun {
    local psout=`ps -u $USER -o comm,pid`
    local runsh=`echo "$psout"|grep ^run.sh`
    echo "$runsh"
    local pids=`echo "$runsh"|awk '{ print $2 }'`
    kill $pids
}
export RUST_BACKTRACE=1
export RUST_SRC_PATH="$(rustc --print sysroot)/lib/rustlib/src/rust/src"
