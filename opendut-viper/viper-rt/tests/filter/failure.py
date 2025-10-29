# VIPER_VERSION = 1.0
from viper import unittest

class MyFailingTestCase(unittest.TestCase):
    def test_failure(self):
        self.fail("BOOM")
