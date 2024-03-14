from __future__ import annotations

import time
import logging

from serial import Serial
from serial.tools.list_ports import comports

MANSON_DEFAULT_BR = 9600
"""Default baudrate for Manson serial ports"""

MANSON_DEVICE_LIST = ((0x10C4, 0xEA60),)
"""List of known USB VID/PID pairs for Manson power supplies"""


class MansonSerialControl:
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
    def open_port(
        cls, port_name: str, br: int = MANSON_DEFAULT_BR, debug=False
    ) -> MansonSerialControl:
        """Open the specified COM port as a Manson serial interface

        :param port_name: The string name of the port. On Windows, this could be `COM4`.
            On Linux, it would be something like `/dev/ttyUSB0`.
        :param br: The baudrate to use for the serial link, the default is
            `MANSON_DEFAULT_BR`.
        :return: A new class instance
        """
        logger = logging.getLogger()
        serial = Serial(port_name, br)

        if debug:
            originalread = serial.read

            def logread(*args, **kwargs):
                data = originalread(*args, **kwargs)
                print(f"read({len(data)}): {data}")
                return data

            serial.read = logread

            originalwrite = serial.write

            def logwrite(data, *args, **kwargs):
                ret = originalwrite(data, *args, **kwargs)
                print(f"write({ret}/{len(data)}): {data}")
                return ret

            serial.write = logwrite

        serial.timeout = 1
        return cls(serial)

    @classmethod
    def open_first_port(
        cls, vid: int = None, pid: int = None, br: int = MANSON_DEFAULT_BR, debug=False
    ) -> MansonSerialControl:
        """Search for a serial port with matching USB VID and PID and open it

        The function searches for available serial ports and checks if their USB
        attributes match a set of known USB vendor IDs / product IDs. It opens the first
        match and returns a new class instance.

        :param vid: The USB Vendor ID to search for. If this parameter is None, a default
            set of VIDs and PIDs is used.
        :param pid: The USB Product ID to search for. If this parameter is None, a default
            set of VIDs and PIDs is used.
        :param br: The baudrate to use for the serial link, the default is
            `MANSON_DEFAULT_BR`.
        :return: A new class instance
        """
        if vid is not None and pid is not None:
            dev_list = [(vid, pid)]
        else:
            dev_list = MANSON_DEVICE_LIST

        for port in comports():
            if (port.vid, port.pid) in dev_list:
                return cls.open_port(port.device, br, debug=debug)

        id_list = [f"{vid:04X}:{pid:04X}" for vid, pid in dev_list]

        raise ValueError(
            "Unable to find Manson with any of these VID / PID: " + ", ".join(id_list)
        )

    def set_power_state(self, power_on: bool) -> None:
        """
        Set the state of the power supply output

        :param power_on: Enable or disable the output of the power supply:
            True -> Enabled, False -> Disabled
        """
        n = "0" if power_on else "1"
        cmd = f"SOUT{n}\r"
        self._serial.write(cmd.encode("ascii"))

        r = self._serial.read(size=3)
        if len(r) < 3:
            raise IOError(f"Device unresponsive")
        if r != b"OK\r":
            raise IOError(f"Incorrect response received from device: {r}")

    def set_voltage(self, voltage):
        # TODO: error handling, if overvoltage
        self._serial.write(f"VOLT{int(voltage*10):03d}\r".encode("ascii"))
        r = self._serial.read(size=3)
        # print(f"{r}")
        if len(r) < 3:
            raise IOError(f"Device unresponsive")
        if r != b"OK\r":
            raise IOError(f"Incorrect response received from device: {r}")

    def get_model(self):
        self._serial.write(f"GMOD\r ".encode("ascii"))
        r = self._serial.read_until(expected=b"\rOK\r")  # (size=9)
        self.model = r[:-4]
        return self.model

    def get_measurements(self):
        """
        Read measurements from hardware.
        Might fail due to flakiness of usb serial -> just retry.


        :return:
        """

        # self._serial.reset_input_buffer()

        # Blocks hardware dials for about 5 seconds. ("REAR CONTROL" LED lights up)

        # VVVVCCCCS\rOK\r

        # self._serial.timeout = 0.5
        self._write_line("GETD")
        r = self._read_line()

        if len(r) != 10:
            raise IOError(f"Incorrect response received from device: {r}")
        # Returns: VVVVCCCCS\rOK\r Four-digit voltage, e.g. 12.34; four-digit current, e.g. 12.34; status 0=constant voltage, 1=constant current.
        voltage = float(r[0:4]) / 100
        current = float(r[4:8]) / 100
        status = int(r[8:9])

        # starttime = time.time()
        r = self._read_line()

        if r != b"OK\r":
            raise IOError(f"Incorrect response received from device: {r}")

        # self._writeline(f"ENDS\r".encode("ascii"))

        return voltage, current, status

    def _write_line(self, command: str):
        self._serial.write(f"{command}\r".encode("ascii"))

    def _read_line(self):
        r = self._serial.read_until(expected=b"\r")
        return r

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
        self._serial.close()
