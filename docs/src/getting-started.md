# Getting started

[Brewer](https://github.com/gmornin/brewer) is a command line utility for interacting with [GM services](https://siriusmart.github.io/gm-services/).

## Installation

Brewer requires no external dependencies. However, you may need to build and install the command using a build system such as [Cargo](https://www.rust-lang.org/tools/install).

```sh
$ cargo install --git https://github.com/gmornin/brewer
```

To confirm Brewer is installed, run the command in terminal.

```sh
$ brewer version
brewer 0.1.0 (git df72cfc) # or similar
```

## Account

This section assumes you already have an account. If not, you might want to [create one](https://gmtex.siri.sh).

```sh
$ brewer login <username> <instance>
```

To see more about a command, run with flag -h.

```sh
$ brewer login -h
```

To see a list of all commands, run -h.

```sh
$ brewer -h
```
