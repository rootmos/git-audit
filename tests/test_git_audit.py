import unittest
import tempfile
import pygit2
import subprocess
import os
import configparser
import json
import random

from . import fresh, w3, faucets, ethereum_rpc_target, normalize

class Content:
    def __init__(self, path, name=None, content=None):
        self.name = name or f"content-{fresh.salt(5)}.txt"
        self.path = os.path.join(path, self.name)
        if not os.path.isfile(self.path):
            self.refresh()

    def refresh(self):
        self.content = fresh.salt(12)

    @property
    def content(self):
        print(f"read: {self.path}")
        with open(self.path, "r") as f:
            return f.read()

    @content.setter
    def content(self, content):
        print(f"write: {self.path} {content}")
        with open(self.path, "w") as f:
            f.write(content)

class test_env:
    def __init__(self, master=None, owner_key=None):
        self.master = master
        self.owner_key = owner_key or master.owner_key if master else faucets.key()
        self.exe = os.getenv("GIT_AUDIT_EXE")

    def __enter__(self):
        self.root = tempfile.TemporaryDirectory()
        self.global_config = os.path.join(self.root.name, "global.json")
        self.path = os.path.join(self.root.name, "repo-" + fresh.salt(5))
        if self.master is None:
            self.repo = pygit2.init_repository(self.path)
        else:
            self.repo = pygit2.clone_repository(self.master.repo.path, self.path)

        self.config = self.master.config if self.master else {
            "ethereum": {
                "private_key": self.owner_key.hex(),
                "rpc_target": ethereum_rpc_target,
                "chain_id": w3.eth.chainId,
            }
        }

        return self

    def __exit__(self, type, value, traceback):
        self.root.cleanup()

    @property
    def config(self):
        return self._config

    @config.setter
    def config(self, value):
        self._config = value

        with open(self.global_config, "w") as f:
            f.write(json.dumps(self._config))

    def run(self, args):
        env = {
            "RUST_LOG": "git_audit",
        }
        if os.getenv("RUST_BACKTRACE") is not None:
            env["RUST_BACKTRACE"] = os.getenv("RUST_BACKTRACE")

        print("executing: ", [self.exe, f"--global-config={self.global_config}"] + args)
        p = subprocess.run(
            [self.exe, f"--global-config={self.global_config}"] + args,
            cwd=self.path,
            env=env,
        )
        if p.returncode == 1: return False
        elif p.returncode == 0: return True
        else: p.check_returncode()

    def file(self, name=None):
        return Content(self.path,
            name = name.name if isinstance(name, Content) else name
        )

    def commit(self, content=None):
        try:
            parents = [self.repo.head.target]
        except:
            parents = []

        tb = self.repo.TreeBuilder()
        for c in content or []:
            b = self.repo.create_blob_fromworkdir(c.name)
            tb.insert(c.name, b, pygit2.GIT_FILEMODE_BLOB)
        t = tb.write()

        return self.repo.create_commit(
            "refs/heads/master",
            fresh.author(), fresh.committer(),
            fresh.commit_msg(),
            t, parents
        )

    def inspect(self):
        return GitAudit(self.path)

    @property
    def commits(self, oid=None):
        return list(map(
            lambda c: c.id.raw.hex(),
            self.repo.walk(oid or self.repo.head.target, pygit2.GIT_SORT_TOPOLOGICAL)
        ))

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
        cs = list(map(lambda c: c.to_bytes(20, "big").hex(), res))
        cs.reverse()
        return cs

class GitAuditTests(unittest.TestCase):
    def test_init_empty_repo(self):
        with test_env() as te:
            te.run(["init"])
            self.assertNotEqual(len(w3.eth.getCode(te.inspect().contract.address)), 0)

    def test_init_non_empty_repo(self):
        with test_env() as te:
            f0 = te.file()
            f1 = te.file()
            te.commit(content=[f0])
            te.run(["init"])

            with test_env(te) as c:
                self.assertEqual(f0.content, c.file(f0).content)
                self.assertNotEqual(f1.content, c.file(f1).content)

    def test_anchor(self):
        with test_env() as te:
            te.run(["init", "--no-commit"])

            te.commit()
            te.run(["anchor"])

            self.assertEqual(te.inspect().commits, te.commits)

    def test_anchor_twice(self):
        with test_env() as te:
            te.run(["init", "--no-commit"])

            te.commit()
            te.run(["anchor"])

            te.commit()
            te.run(["anchor"])

            self.assertEqual(te.inspect().commits, te.commits)

    def test_anchor_in_downstream(self):
        with test_env() as te0:
            te0.run(["init"])

            with test_env(te0) as te1:
                te1.run(["anchor"])

    def test_validate_empty_repo(self):
        with test_env() as te:
            te.run(["init"])
            te.run(["validate"])

    def test_validate_non_empty_repo(self):
        with test_env() as te:
            te.run(["init"])
            for _ in range(1, random.randint(1, 10)): te.commit()
            te.run(["validate"])

    def test_validate_cloned_repo(self):
        with test_env() as te0:
            te0.run(["init"])
            te0.run(["anchor"])

            with test_env(te0) as te1:
                te1.run(["validate"])

    def test_validate_reject(self):
        with test_env() as te0:
            te0.run(["init"])

            with test_env(te0) as te1:
                # anchor a commit in the upstream repo
                te0.commit()
                te0.run(["anchor"])

                # not present in downstream repo => validation failure
                self.assertFalse(te1.run(["validate"]))

    def test_validate_reject(self):
        with test_env() as te0:
            te0.run(["init"])

            with test_env(te0) as te1:
                # anchor a commit in the downstream repo
                te1.commit()
                te1.run(["anchor"])

            # not present in upstream repo => validation failure
            self.assertFalse(te0.run(["validate"]))
