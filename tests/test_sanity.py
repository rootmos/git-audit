import unittest
import tempfile
import pygit2
import random

from . import fresh, w3, faucets, to_address

class SanityCheckGit(unittest.TestCase):
    def test_commit(self):
        with tempfile.TemporaryDirectory() as d:
            r = pygit2.init_repository(d)
            tree = r.index.write_tree()
            c0 = r.create_commit(
                "refs/heads/master",
                fresh.author(), fresh.committer(),
                fresh.commit_msg(),
                tree, []
            )

            [c1] = r.walk(r.head.target, pygit2.GIT_SORT_TOPOLOGICAL)
            self.assertEqual(c0, c1.id)


class SanityCheckEtherum(unittest.TestCase):
    def test_blockNumber(self):
        w3.eth.blockNumber

    def test_faucets(self):
        for f in faucets.addresses:
            b = w3.eth.getBalance(f)
            self.assertGreater(b, 0)

    def test_faucets_present_in_rpc(self):
        for i, f in enumerate(faucets.addresses):
            self.assertEqual(f, w3.eth.accounts[i])

    def test_fresh_private_key(self):
        pk = fresh.private_key()
        b = w3.eth.getBalance(to_address(pk))
        self.assertEqual(b, 0)

    def test_fresh_address(self):
        b = w3.eth.getBalance(fresh.address())
        self.assertEqual(b, 0)

    def test_fresh_private_key_with_balance(self):
        w = w3.toWei(random.randint(1, 1000), 'gwei')
        pk = fresh.private_key(balance = w)
        b = w3.eth.getBalance(to_address(pk))
        self.assertEqual(b, w)
