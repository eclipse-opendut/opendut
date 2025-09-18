use indoc::indoc;

pub const SCRIPT_PY_TEMPLATE: &str = indoc! {"
    # VIPER_VERSION = 1.0
    from viper import unittest

    class MyTestCase(unittest.TestCase):
        def test_hello(self):
            self.assertEquals(True, True)
"};
