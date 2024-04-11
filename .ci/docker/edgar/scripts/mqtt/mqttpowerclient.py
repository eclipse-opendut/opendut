#!/usr/bin/python
import paho.mqtt.client as mqtt
import argparse
import manson
import threading
from tenacity import Retrying, stop_after_attempt
import time
import logging

logging.getLogger().setLevel("DEBUG")
# setup synchronization and retrying


class MqttSerialControl:

    _model = None
    _voltage = None
    _current = None
    _power_state = None

    def on_connect(self, client, userdata, flags, rc, unsure):
        logging.info("Connected with result code " + str(rc))
        client.subscribe(self._localtopic + "/model", qos=1)
        client.subscribe(self._localtopic + "/0/voltage", qos=1)
        client.subscribe(self._localtopic + "/0/current", qos=1)

    # client.subscribe(localtopic + "/update_interval",qos=1)

    def on_model(self, client, userdata, message):
        self._model = message.payload.decode()
        self._model_available.set()
        logging.info(f"received model {self._model}")

    def on_voltage(self, client, userdata, message):
        self._voltage = float(message.payload.decode())
        self._voltage_available.set()
        logging.info(f"received voltage {self._voltage}")

    def on_current(self, client, userdata, message):
        self._current = float(message.payload.decode())
        self._current_available.set()
        logging.info(f"received current {self._current}")

    def on_message(self, client, userdata, msg):
        logging.debug(
            "incoming messages: topic:" + msg.topic + " payload:" + str(msg.payload)
        )

    def __init__(self, mqtt_server, mqtt_port, localtopic):
        """
        Remote control of a power supply via MQTT

        Note: The class instance will automatically close the bound serial port when it
        goes out of scope.

        :param
        """

        self._localtopic = localtopic
        self._mqtt_server = mqtt_server
        self._mqtt_port = mqtt_port
        # Generate a unique ID, that is however persistent between restarts.
        self._client_id = "testclient_" + localtopic.replace("/", "_")

        self._client = mqtt.Client(client_id=self._client_id, protocol=mqtt.MQTTv5)
        self._client.on_connect = self.on_connect
        self._client.on_message = self.on_message

        self._client.message_callback_add(localtopic + "/0/voltage", self.on_voltage)
        self._voltage_available = threading.Event()
        self._client.message_callback_add(localtopic + "/0/current", self.on_current)
        self._current_available = threading.Event()
        self._client.message_callback_add(localtopic + "/model", self.on_model)
        self._model_available = threading.Event()
        # client.message_callback_add(localtopic + "/update_interval", on_update_interval)
        logging.info(f"Connecting to server {mqtt_server}:{mqtt_port}...")
        self._client.connect(self._mqtt_server, mqtt_port, 60)

        logging.info(f"{self._model=}")
        # client.publish(topic=localtopic + "/model", payload=model, qos=2, retain=False)

        self._client.loop_start()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    def get_model(self):
        return self._model

    def set_power_state(self, power_on: bool) -> None:
        """
        Set the state of the power supply output

        :param power_on: Enable or disable the output of the power supply:
            True -> Enabled, False -> Disabled
        """
        if power_on == True:
            payload = 1
        elif power_on == False:
            payload = 0
        msginfo = self._client.publish(
            topic=self._localtopic + "/0/switch", payload=str(payload), qos=1, retain=True
        )
        msginfo.wait_for_publish()

    def set_voltage(self, voltage: float, retry=False):
        msginfo = self._client.publish(
            topic=self._localtopic + "/0/voltage_target",
            payload=voltage,
            qos=1,
            retain=True,
        )
        msginfo.wait_for_publish()

    def get_measurements(self, timeout=None):
        # TODO fixme maybe wait for things at the same time
        if not timeout is None:
            self._voltage_available.wait(timeout)
        voltage = self._voltage
        if not timeout is None:
            self._current_available.wait(timeout)
        current = self._current
        status = True
        # TODO check how old?
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
        self._client.loop_stop()
        pass


# client.publish(topic=localtopic + "/update_interval", payload=update_interval, qos=2, retain=False)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="Powersupply MQTT Client",
        description="connects to powersupply via MQTT",
    )
    parser.add_argument("--server", type=str, default="localhost")
    parser.add_argument("--port", type=int, default=1884)
    parser.add_argument("--topic", default="defaultpeer/defaultpowersupply")
    parser.add_argument("--interval", type=float, default=0.5)

    parser.add_argument("--voltage", type=float, default=None)
    parser.add_argument("--poweron", action='store_true')
    parser.add_argument("--poweroff", action='store_true')
    parser.add_argument("--powercycle", action='store_true')
    parser.add_argument("--getmeasurement", action='store_true')
    args = parser.parse_args()
    ps = MqttSerialControl(args.server, args.port, args.topic)
    if not args.voltage is None:
        ps.set_voltage(args.voltage)

    if args.poweron:
        ps.set_power_state(True)

    if args.poweroff:
        ps.set_power_state(False)

    if args.powercycle:
        ps.set_power_state(False)
        time.sleep(1)
        ps.set_power_state(True)

    if args.getmeasurement:
        voltage, current, status = ps.get_measurements(timeout=10)
        print(f"{voltage},{current}")