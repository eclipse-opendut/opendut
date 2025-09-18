# VIPER_VERSION = 1.0
from viper import unittest
import sys

class MyTestCase(unittest.TestCase):
    def test_awesomeness(self):
        print("Awesome!")
        eprint("Awesome Err")
    
    def eprint(*args, **kwargs):
        print(*args, file=sys.stderr, **kwargs)
