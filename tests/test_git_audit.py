import unittest
import tempfile
import pygit2
import subprocess
import os
import configparser
import json
import random

from . import fresh, w3, faucets, ethereum_rpc_target, normalize

class test_env:
    def __enter__(self):
        self.exe = os.getenv("GIT_AUDIT_EXE")
        self.root = tempfile.TemporaryDirectory()
        self.global_config = os.path.join(self.root.name, "global.json")
        self.repo = pygit2.init_repository(os.path.join(self.root.name, "repo"))

        self.config = {
            "ethereum": {
                "private_key": faucets.key().hex(),
                "rpc_target": ethereum_rpc_target,
                "chain_id": w3.eth.chainId,
            }
        }

        with open(self.global_config, "w") as f:
            f.write(json.dumps(self.config))

        return self

    def __exit__(self, type, value, traceback):
        self.root.cleanup()

    def run(self, args):
        subprocess.check_call(
            [self.exe, f"--global-config={self.global_config}"] + args,
            cwd=self.repo.path,
            env={
                "RUST_LOG": "git_audit",
                "RUST_BACKTRACE": os.getenv("RUST_BACKTRACE", default=""),
            }
        )

    def commit(self):
        try:
            parents = [self.repo.head.target]
        except:
            parents = []

        return self.repo.create_commit(
            "refs/heads/master",
            fresh.author(), fresh.committer(),
            fresh.commit_msg(),
            self.repo.index.write_tree(), parents
        )

    def inspect(self):
        return GitAudit(self.repo.path)

class GitAudit:
    def __init__(self, path):
        self.path = path

        with open(os.path.join(path, ".git-audit.json")) as f:
            c = json.loads(f.read())

        self.contract = w3.eth.contract(
            address = normalize(c["contract"]["address"]),
            abi = c["contract"]["abi"],
        )

    @property
    def commits(self):
        res = self.contract.functions.commits().call()
        return list(map(lambda c: c.to_bytes(20, "big"), res))

class GitAuditTests(unittest.TestCase):
    def test_init(self):
        with test_env() as te:
            te.run(["init"])
            self.assertNotEqual(len(w3.eth.getCode(te.inspect().contract.address)), 0)

    def test_anchor(self):
        with test_env() as te:
            te.run(["init"])

            c0 = te.commit()
            te.run(["anchor"])

            self.assertEqual(te.inspect().commits, [c0.raw])

    def test_anchor_twice(self):
        with test_env() as te:
            te.run(["init"])

            c0 = te.commit()
            te.run(["anchor"])

            c1 = te.commit()
            te.run(["anchor"])

            self.assertEqual(te.inspect().commits, [c0.raw, c1.raw])

    def test_validate_empty_repo(self):
        with test_env() as te:
            te.run(["init"])
            te.run(["validate"])

    def test_validate(self):
        with test_env() as te:
            te.run(["init"])
            te.commit()
            te.run(["validate"])
