# SigMF Utilities

Some command-line utilities to manipulate SigMF compliant files:

* [sigmf-col](#sigmf-collection)
* [sigmf-hash](#sigmf-hash)

## SigMF Hash

Check and update hashes on sigmf files

Usage: ```sigmf-hash <COMMAND> <FILES>..```

Commands:

* check   Verify the hash of a dataset
* update  Recompute and update the hash of a dataset

Examples:

```sigmf-hash check samples/test1```

```sigmf-hash update samples/test1```

## SigMF Collection

Create and updates collection of SigMF records

Usage: ```sigmf-col <COMMAND>```

Commands:

* create  Create a collection from given SigMF files
* ~~update  Update a collection~~
* help    Print this message or the help of the given subcommand(s)

Examples:

```sigmf-col create -o samples/index.sigmf-meta samples/*.sigmf-data```
