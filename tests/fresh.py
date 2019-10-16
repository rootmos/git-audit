import random
import string
import pygit2
import web3

from . import faucets, to_address

def bs(n):
    return bytes(random.getrandbits(8) for _ in range(n))

def coin_flip():
    return random.choice([True, False])

def salt(n=5):
    alphabeth = string.ascii_lowercase
    return ''.join(random.choice(alphabeth) for i in range(n))

def author():
    return pygit2.Signature("Foo the author", "foo@authors.test")

def committer():
    return pygit2.Signature("Bar the committer", "bar@committers.test")

def commit_msg():
    return "Hello committed world!"

def mether():
    return web3.Web3.toWei(random.randint(1, 1000), 'milli');

def private_key(balance = None):
    pk = bs(32)
    if balance: faucets.ether(to_address(pk), balance)
    return pk

def address():
    return to_address(private_key())
