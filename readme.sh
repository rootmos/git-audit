#!/bin/bash

set -o nounset -o pipefail -o errexit

GIT_AUDIT_EXE=${GIT_AUDIT_EXE-./git-audit}

cat <<EOF
# git-audit

\`\`\`
$($GIT_AUDIT_EXE --help)
\`\`\`
EOF
