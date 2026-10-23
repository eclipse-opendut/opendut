# VIPER_VERSION = 1.0
from viper import parameters, unittest

LETTER = parameters.TextParameter("letter", max=1)
VERSION = parameters.NumberParameter("version")
ENABLED = parameters.BooleanParameter("enabled", default=False)

class TextParameter(unittest.TestCase):
    def test_if_letter_is_a(self):
        self.assertEquals(self.parameters.get(LETTER), "a")

    def test_if_letter_is_b(self):
        self.assertEquals(self.parameters.get(LETTER), "b")

class NumberParameter(unittest.TestCase):
    def test_if_version_is_1(self):
        self.assertEquals(self.parameters.get(VERSION), 1)

class BooleanParameter(unittest.TestCase):
    def test_if_enabled(self):
        self.assertTrue(self.parameters.get(ENABLED))
