# VIPER_VERSION = 1.0
from viper import unittest

class MySucceedingTestCase(unittest.TestCase):
    def test_success(self):
        print("Success!")
