import unittest
import random

from . import w3, fresh
from .test_env import test_env

class InitTests(unittest.TestCase):
    def test_init_commit(self):
        with test_env() as te:
            [] = te.commits
            te.run(["init"])
            [c] = map(lambda c: te.repo[c], te.commits)
            self.assertEqual(c.message, "Initializing git-audit")
            self.assertEqual((c.author.name, c.author.email), ("git-audit", "git-audit@rootmos.io"))
            self.assertEqual(c.author, c.committer)
            self.assertNotEqual(len(w3.eth.getCode(te.inspect().contract.address)), 0)

    def test_init_empty_repo(self):
        with test_env() as te:
            te.run(["init", "--no-commit"])
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

    def test_init_outside_repository(self):
        with test_env() as te:
            te.run([f"--repository={te.path}", "init"], cwd=te.root)
            te.inspect()

class AnchorTests(unittest.TestCase):
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

class ValidateTests(unittest.TestCase):
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
            te0.run(["validate"], expect_exit_code=1)

class SecurityTests(unittest.TestCase):
    def test_ownership(self):
        with test_env() as te0:
            te0.run(["init"])

            with test_env(te0, owner_key=fresh.private_key(fresh.mether())) as te1:
                te1.run(["anchor"])
                self.assertNotEqual(te1.inspect().commits, te0.commits)

class ErrorReportingTests(unittest.TestCase):
    def test_init_outside_repository(self):
        with test_env() as te:
            (stdout, stderr) = te.run(["init"], cwd=te.root, capture_output=True, expect_exit_code=1)
            self.assertEqual(stdout, b"")
            self.assertEqual(stderr, b"unable to open a git repository at: .\n")

    def test_init_twice(self):
        with test_env() as te:
            te.run(["init"])
            (stdout, stderr) = te.run(["init"], capture_output=True, expect_exit_code=1)
            self.assertEqual(stdout, b"")
            ca = te.inspect().contract.address.lower()
            print(ca)
            print(stderr)
            self.assertEqual(stderr, f"repository is already initialized and anchored to contract: {ca}\n".encode("UTF-8"))

    def test_anchor_in_uninitialized_repo(self):
        with test_env() as te:
            (stdout, stderr) = te.run(["anchor"], capture_output=True, expect_exit_code=1)
            self.assertEqual(stdout, b"")
            self.assertEqual(stderr, b"repository isn't initialized\n")

    def test_validate_in_uninitialized_repo(self):
        with test_env() as te:
            (stdout, stderr) = te.run(["validate"], capture_output=True, expect_exit_code=1)
            self.assertEqual(stdout, b"")
            self.assertEqual(stderr, b"repository isn't initialized\n")
