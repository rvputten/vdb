#!/bin/bash
progname=$(basename $(pwd))
export DISPLAY=:0
export RUST_BACKTRACE=1

lines=${LINES:-89}

# test, release, debug
mode=test

inotifywait -e close_write src resources run.sh 2> /dev/null || { echo "Error running inotifywait"; exit 1; }
sleep 0.1

function rqline {
    local i=0;
    local c=${COLUMNS:-80}
    local title="$1"

    printf "\e[92m"

    if [ ! -z "$title" ]; then
	c=$((c-${#title}-4))
	printf -- "-- $title "
    fi

    while [[ $i -lt $c ]]; do
        printf "-";
        i=$((i+1));
    done;
    echo -e "\e[0m"
}

function run_cargo {
    local cmd=$1; shift
    rqline $cmd

    out="`cargo $cmd $* --color=always 2>&1`"
    local r=$?
    echo "$out" | head -$((lines-5))
    return $r

#    cargo $cmd $* --color=always
#    local r=$?
#    return $r
}

rqline

case $mode in
    clippy)
	run_cargo clippy
	;;
    test)
	cargo fmt
	run_cargo clippy &&
	    run_cargo test &&
	    cargo build > /dev/null 2>&1 &&
	    false &&
	    rqline run &&
	    cargo run | head -48
	;;
    release)
	cargo build --release && (
	    killall $progname
	    target/release/$progname ~/out.csv &
	)
	;;
    debug)
	cargo build && (
	    killall $progname
	    target/debug/$progname ~/out.csv &
	)
	;;
esac

sleep 0.5
./run.sh &
