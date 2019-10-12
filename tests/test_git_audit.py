import unittest
import tempfile
import pygit2
import subprocess
import os
import configparser
import json

from . import fresh, w3, faucets, ethereum_rpc_target, normalize

def run(args, cwd):
    subprocess.check_call(
        [os.getenv("GIT_AUDIT_EXE")] + args,
        cwd=cwd,
        env={
            "RUST_LOG": "debug",
            "RUST_BACKTRACE": "1",
        }
    )

def mk_global_config(d):
    c = {
        "ethereum": {
            "private_key": faucets.key().hex(),
            "rpc_target": ethereum_rpc_target,
            "chain_id": w3.eth.chainId,
        }
    }
    p = os.path.join(d, "global.json")
    with open(p, "w") as f:
        f.write(json.dumps(c))
    return p

class GitAuditTests(unittest.TestCase):
    def test_init(self):
        with tempfile.TemporaryDirectory() as d:
            gc = mk_global_config(d)
            r = pygit2.init_repository(os.path.join(d, "repo"))
            run([f"--global-config={gc}", "init"], cwd=r.path)

            with open(os.path.join(r.path, ".git-audit.json")) as f:
                c = json.loads(f.read())

            contract = normalize(c["contract"])
            self.assertNotEqual(len(w3.eth.getCode(contract)), 0)
