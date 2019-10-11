import unittest
import tempfile
import pygit2

from . import fresh

class SanityChecks(unittest.TestCase):
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
