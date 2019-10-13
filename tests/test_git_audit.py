import unittest
import tempfile
import pygit2
import subprocess
import os
import configparser
import json
import random

from . import fresh, w3, faucets, ethereum_rpc_target, normalize

def run(args, cwd):
    subprocess.check_call(
        [os.getenv("GIT_AUDIT_EXE")] + args,
        cwd=cwd,
        env={
            "RUST_LOG": "git_audit",
            "RUST_BACKTRACE": os.getenv("RUST_BACKTRACE", default=""),
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

            contract = normalize(c["contract"]["address"])
            self.assertNotEqual(len(w3.eth.getCode(contract)), 0)

    def test_anchor(self):
        with tempfile.TemporaryDirectory() as d:
            gc = mk_global_config(d)
            r = pygit2.init_repository(os.path.join(d, "repo"))
            tree = r.index.write_tree()
            run([f"--global-config={gc}", "init"], cwd=r.path)

            with open(os.path.join(r.path, ".git-audit.json")) as f:
                c = json.loads(f.read())

            contract = w3.eth.contract(
                address = normalize(c["contract"]["address"]),
                abi = c["contract"]["abi"],
            )

            c0 = r.create_commit(
                "refs/heads/master",
                fresh.author(), fresh.committer(),
                fresh.commit_msg(),
                tree, []
            )

            run([f"--global-config={gc}", "anchor"], cwd=r.path)

            cs = list(map(lambda c: c.to_bytes(20, "big"),
                contract.functions.commits().call()))

            self.assertEqual(cs, [c0.raw])
