import os
import web3
import random
from eth_keys import keys
from binascii import unhexlify

ethereum_rpc_target = os.getenv("ETHEREUM_RPC_TARGET")
w3 = web3.Web3(web3.HTTPProvider(ethereum_rpc_target))

def ether(v):
    return web3.Web3.toWei(v, 'ether');

def to_address(pk):
    if not isinstance(pk, keys.PrivateKey):
        pk = keys.PrivateKey(pk)
    return pk.public_key.to_checksum_address()

def normalize(a):
    return web3.Web3.toChecksumAddress(a)

class Faucets:
    def __init__(self):
        self.keys = list(map(unhexlify, [
            "e2ee547be17ac9f7777d4763c43fd726c0a2a6d40450c92de942d7925d620b6d",
            "0740fb09781e8fa771edcf1bddee93ad6772593b3139f1cf36b0d095d235887b",
            "ac72e464dac0448a28fa71b34bfe46b2356fe09bd4f5a73519ee60b3b92b9dab",
            "230eda6cc73da415d3b327426dde475a786bb5a0aeae2ca531aaaa8c0218a7a5",
            "91e3179925ef60e4d1f4daf0e7d67bdb5cf74ff3d456db0eb239e432290db31c",
            "66769c67a372926b945262a1c86b7944a669dbeab3d89771d7af691b3bfb20d8",
            "af40a15c4d369cdb39d01148d7b5f5dd4f9825447fabcbfc15e230db84fcb88b",
            "4ad882b7e0b24fd01ad6d2f281d469edb9d2bef2c2ee8871099c5fd7c7018317",
            "9042fc069b6abe8210d31195b382b61c3ee9149223fcb181016a49ba61a14d84",
            "bf32730f2b240c0c482126ecc1e2219554f3c738f19bd592e3ccf4cc005ddc1e",
        ]))

        self.addresses = list(map(to_address, self.keys))

    def address(self):
        return random.choice(self.addresses)

    def key(self):
        return random.choice(self.keys)

    def ether(self, to, amount):
        w3.eth.sendTransaction({
            "from": self.address(),
            "to": to,
            "value": amount,
        })

faucets = Faucets()
