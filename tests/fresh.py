import random
import string
import pygit2

from . import faucets, to_address

def bs(n):
    return bytes(random.getrandbits(8) for _ in range(n))

def salt(n=5):
    alphabeth = string.ascii_lowercase
    return ''.join(random.choice(alphabeth) for i in range(n))

def author():
    return pygit2.Signature("Foo the author", "foo@authors.test")

def committer():
    return pygit2.Signature("Bar the committer", "bar@committers.test")

def commit_msg():
    return "Hello committed world!"

def private_key(balance = None):
    pk = bs(32)
    if balance: faucets.ether(to_address(pk), balance)
    return pk

def address():
    return to_address(private_key())
