import random
import pygit2

def author():
    return pygit2.Signature("Foo the author", "foo@authors.test")

def committer():
    return pygit2.Signature("Bar the committer", "bar@committers.test")

def commit_msg():
    return "Hello committed world!"
