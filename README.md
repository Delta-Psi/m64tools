# m64tools
Super Mario 64 music format tools

Currently includes the following executables:

## inspect\_aiff
Run with `cargo run inspect_aiff < [AIFF file]`. This reads an AIFF file from standard input and
prints information about its contents. Located [here](aiff/src/bin/inspect_aiff.rs).

## aiffplay
Run with `cargo run aiffplay < [AIFF file]`. This reads an AIFF file from standard input and
plays it back. Located [here](aiffplay/src/main.rs).

## m64play
Run with `cargo run m64play < [m64 file]`. This reads a m64 file from standard input and attempts
to play it back. Note that it's presently in an incredibly barebones state, and will, for example,
not load the actual samples. Located [here](m64play/src/main.rs).
