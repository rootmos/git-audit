# git-audit

[![Build Status](https://travis-ci.org/rootmos/git-audit.svg?branch=master)](https://travis-ci.org/rootmos/git-audit)

```
git-audit 0.1.0
Gustav Behm <me@rootmos.io>
Manages an audit trail for a Git repository by considering it as an Ethereum side-chain

USAGE:
    git-audit [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -g, --global-config <global-config>    
    -r, --repository <repository>          

SUBCOMMANDS:
    anchor      Anchors a commit in the audit trail
    help        Prints this message or the help of the given subcommand(s)
    init        Deploys a Ethereum smart contract to collect the audit trail
    validate    Validates the audit trail
```
