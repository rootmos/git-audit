#!/bin/bash

set -o nounset -o pipefail -o errexit

GIT_AUDIT_EXE=${GIT_AUDIT_EXE-./git-audit}

cat <<EOF
# git-audit

[![Build Status](https://travis-ci.org/rootmos/git-audit.svg?branch=master)](https://travis-ci.org/rootmos/git-audit)

\`\`\`
$($GIT_AUDIT_EXE --help)
\`\`\`
EOF
