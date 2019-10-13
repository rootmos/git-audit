import unittest
import tempfile
import pygit2
import random
import os

from . import fresh, w3, faucets, to_address

class SanityCheckGit(unittest.TestCase):
    def commit(self, r):
        try:
            parents = [r.head.target]
        except:
            parents = []

        return r.create_commit(
            "refs/heads/master",
            fresh.author(), fresh.committer(),
            fresh.commit_msg(),
            r.index.write_tree(), parents,
        )

    def walk(self, r, oid=None):
        return list(map(
            lambda c: c.id,
            r.walk(oid or r.head.target, pygit2.GIT_SORT_TOPOLOGICAL)
        ))

    def test_commit(self):
        with tempfile.TemporaryDirectory() as d:
            r = pygit2.init_repository(d)
            c0 = self.commit(r)
            self.assertEqual([c0], self.walk(r))

    def test_two_commits(self):
        with tempfile.TemporaryDirectory() as d:
            r = pygit2.init_repository(d)
            c0 = self.commit(r)
            c1 = self.commit(r)
            self.assertEqual([c1, c0], self.walk(r))

    def test_clone(self):
        with tempfile.TemporaryDirectory() as d:
            r0 = pygit2.init_repository(os.path.join(d, "upstream"))
            c0 = self.commit(r0)
            r1 = pygit2.clone_repository(r0.path, os.path.join(d, "downstream"))
            self.assertEqual([c0], self.walk(r1))
            c1 = self.commit(r1)
            self.assertEqual([c0], self.walk(r0))
            self.assertEqual([c1, c0], self.walk(r1))

    def test_push(self):
        with tempfile.TemporaryDirectory() as d:
            r0 = pygit2.init_repository(os.path.join(d, "upstream"), bare=True)
            c0 = self.commit(r0)
            r1 = pygit2.clone_repository(r0.path, os.path.join(d, "downstream"))
            self.assertEqual([c0], self.walk(r1))
            c1 = self.commit(r1)
            r1.remotes["origin"].push(["refs/heads/master"])
            self.assertEqual([c1, c0], self.walk(r0))
            self.assertEqual([c1, c0], self.walk(r1))

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
