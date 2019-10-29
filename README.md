# Corg

A command line tool for literate devops.

## Description

Corg turns markdown files into runnable shell scripts and provides a few useful mechanisms for deploying those scripts.

## Commands

### Convert

Convert a markdown document to a shell script.

```shell
corg --convert path/to/file.md
```
### Run

Deploy a script to a remote host.

```shell
corg run --host faye.futuregadgetlab.dev --script examples/system/nix.sh
```

### Help

View help for the Corg command itself or its subcommands

```shell
corg run --host faye.futuregadgetlab.dev --script examples/system/nix.sh
```
