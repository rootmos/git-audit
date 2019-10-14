import unittest
import random

from . import w3
from .test_env import test_env

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
