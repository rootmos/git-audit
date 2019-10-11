import unittest
import tempfile
import pygit2
import subprocess
import os
import configparser

from . import fresh

def run(args, cwd):
    subprocess.check_call([os.getenv("GIT_AUDIT_EXE")] + args, cwd=cwd)

class GitAuditTests(unittest.TestCase):
    def test_init(self):
        with tempfile.TemporaryDirectory() as d:
            run(["init"], cwd=d)

            with open(os.path.join(d, ".git-audit")) as f:
                c = configparser.ConfigParser()
                c.read_string("[DEFAULT]\n" + f.read())

            self.assertEqual(c['DEFAULT']["foo"], "bar")
