from __future__ import annotations

import time
import random


class MockSerialControl:
    def __init__(self, serial: Serial):
        """
        Remote control of a Manson 3304 power supply via Serial UART

        This class provides a simple interface for controlling the Manson 3304 power
        supply via the Serial UART port. You can either use the constructor and supply
        it a preconstructed serial interface, or use the convenience
        method as follows:

        Example::
            >>> with MansonSerialControl.open_port("COM4") as ctrl:
            >>>     ctrl.power_cycle(wait_time_s=0.5)

        The class can also search for available interfaces::
            >>> with MansonSerialControl.open_first_port() as ctrl:
            >>>     ctrl.power_cycle(wait_time_s=0.5)

        Note: The class instance will automatically close the bound serial port when it
        goes out of scope.

        :param serial: The open serial port to use for communication with the power supply
        """
        self._serial = serial

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    @classmethod
    def open_port(cls, port_name: str, br: int = 9600):
        return cls(port_name)

    @classmethod
    def open_first_port(cls, vid: int = None, pid: int = None, br: int = 9000):
        return cls.open_port(True, br)

    def get_model(self):
        return "Mock-Powersupply"

    def set_power_state(self, power_on: bool) -> None:
        """
        Set the state of the power supply output

        :param power_on: Enable or disable the output of the power supply:
            True -> Enabled, False -> Disabled
        """
        n = "0" if power_on else "1"

    def set_voltage(self, voltage, retry=False):
        pass

    def get_measurements(self):
        voltage = random.gauss(12.3456, 0.1)
        current = random.gauss(0.123, 0.1)
        status = True
        time.sleep(0.5)
        return voltage, current, status

    def power_cycle(self, wait_time_s: float, post_wait_time_s: float = None) -> None:
        """
        Cycle the power supply output, i.e. turn it off and on again

        :param wait_time_s: The wait time in seconds between turning it off and on again
        :param post_wait_time_s: The wait time after turning power back on
        """
        self.set_power_state(power_on=False)
        time.sleep(wait_time_s)
        self.set_power_state(power_on=True)

        if post_wait_time_s is not None:
            time.sleep(post_wait_time_s)

    def close(self):
        pass
