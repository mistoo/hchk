# healthchecks.io command line client

A CLI interface to healthchecks.io written in Rust

# Usage

$ hchk help
    USAGE:
    hchk [FLAGS] [SUBCOMMAND]

    FLAGS:
    -h, --help       Prints help information
    -v               be verbose
    -V, --version    Prints version information

    SUBCOMMANDS:
    add      Add check
    del      Delete check
    ls       List checks
    pause    Pause check
    ping     Ping check
    setkey   Save API key to $HOME/.hchk
    help     Prints this message or the help of the given subcommand(s)

Save healthchecks.io API key to `$HOME/.hchk`

    $ hchk setkey YOUR_API_KEY

Add new check:

    $ hchk add check-name "30 10 * * *"

Delete check:

    $ hchk del check-name

Ping:

    $ hchk ping check-name

Pause:

    $ hchk pause check-name

List all checks:

    $ hchk ls

List `down` checks:

    $ hchk ls -d

"Long" listing checks:

    $ hchk ls -l

# Build

$ cargo build --release
