import os
import tempfile
import pygit2
import json
import subprocess
import socket
import threading

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

class test_env:
    def __init__(self, master=None, owner_key=None):
        self.master = master
        self.owner_key = owner_key or master.owner_key if master else faucets.key()
        self.exe = os.getenv("GIT_AUDIT_EXE")

    def __enter__(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = self.tmp.name
        self.global_config = os.path.join(self.root, "global.json")
        self.path = os.path.join(self.root, "repo-" + fresh.salt(5))
        if self.master is None:
            self.repo = pygit2.init_repository(self.path)
        else:
            self.repo = pygit2.clone_repository(self.master.repo.path, self.path)

        self.log_file_path = os.path.join(self.root, "log")
        self.log_thread = threading.Thread(target=self._log_reader)
        self.log_thread.start()

        self.config = {
            "ethereum": {
                "private_key": self.owner_key.hex(),
                "rpc_target": ethereum_rpc_target,
                "chain_id": w3.eth.chainId,
            },
            "logging": {
                "file": self.log_file_path,
            },
        }

        return self

    def __exit__(self, type, value, traceback):
        self.tmp.cleanup()
        self.log_thread.join()

    def _log_reader(self):
        with socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM) as s:
            s.bind(self.log_file_path)
            s.settimeout(0.2)
            while os.path.exists(self.log_file_path):
                try:
                    bs, _ = s.recvfrom(4096)
                    self._log_entry(json.loads(bs))
                except socket.timeout:
                    pass

    def _log_entry(self, l):
        if l["level"]["numeric"] < 2 or l["target"] == "git_audit":
            print(f"{l['time']} {l['level']['label']} {l['message']}")

    @property
    def config(self):
        return self._config

    @config.setter
    def config(self, value):
        self._config = value

        with open(self.global_config, "w") as f:
            f.write(json.dumps(self._config))

    def run(self, args, expect_exit_code=0, capture_output=False, cwd=None):
        env = {
            "RUST_LOG": "git_audit",
        }
        if os.getenv("RUST_BACKTRACE") is not None:
            env["RUST_BACKTRACE"] = os.getenv("RUST_BACKTRACE")

        print("executing: ", [self.exe, f"--global-config={self.global_config}"] + args)

        p = subprocess.run(
            [self.exe, f"--global-config={self.global_config}"] + args,
            cwd=cwd or self.path,
            env=env,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )

        if expect_exit_code == 0:
            try:
                p.check_returncode()
            except subprocess.CalledProcessError as e:
                print(p.stdout.decode("UTF-8"))
                print(p.stderr.decode("UTF-8"))
                raise e
        else:
            assert(expect_exit_code == p.returncode)

        if capture_output:
            return (p.stdout, p.stderr)
        else:
            assert p.stdout == b''
            assert p.stderr == b''

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
