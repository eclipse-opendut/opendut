#!/usr/bin/python
import paho.mqtt.client as mqtt
import argparse
import manson
from threading import Lock
from tenacity import Retrying, stop_after_attempt
import time
import logging

logging.getLogger().setLevel("DEBUG")
parser = argparse.ArgumentParser(
    prog="Powersupply MQTT-Daemon",
    description="connects a Manson powersupply to an MQTT server",
)
parser.add_argument("--server")
parser.add_argument("--port", type=int, default=1884)
parser.add_argument("--topic", default="defaultpeer/defaultpowersupply")
parser.add_argument("--interval", type=float, default=0.5)
# TODO
parser.add_argument("--device", default="first")
args = parser.parse_args()

# setup synchronization and retrying
lock = Lock()
retryer = Retrying(stop=stop_after_attempt(5), reraise=True)

# setup args
# Generate a unique ID, that is however persistent between restarts.
client_id = "powersupply_" + args.topic.replace("/", "_")
update_interval = args.interval
localtopic = args.topic

if args.device == "mock":
    import mockpower

    powersupply = mockpower.MockSerialControl.open_first_port()
elif args.device == "first":
    powersupply = manson.MansonSerialControl.open_first_port()
else:
    powersupply = manson.MansonSerialControl.open_port(args.device)


def on_connect(client, userdata, flags, rc, unsure):
    logging.info("Connected with result code " + str(rc))

    client.subscribe(localtopic + "/0/switch", qos=1)
    client.subscribe(localtopic + "/0/voltage_target", qos=1)
    # client.subscribe(localtopic + "/update_interval",qos=1)


def on_update_interval(client, userdata, message):
    i = float(message.payload.decode())
    if i > 0 and i < 10:
        update_interval = i


def on_voltage(client, userdata, message):
    u = float(message.payload.decode())
    logging.info(f"Set voltage {u}")
    lock.acquire()
    try:
        retryer(powersupply.set_voltage, u)
    except:
        logging.exception(f"failed to set voltage {u}")
        pass
    finally:
        logging.debug("release lock")
        lock.release()


def on_switch(client, userdata, message):
    mpowerstate = message.payload.decode()
    logging.debug(f'Message Received for switch: "{mpowerstate}"')
    mpowerstate = int(mpowerstate)
    if mpowerstate == 1:
        powerstate = True
    elif mpowerstate == 0:
        powerstate = False
    else:
        logging.error(f'Invalid power state request:"{mpowerstate}"')
        return
    print(f"State: {powerstate}")
    lock.acquire()
    retryer(powersupply.set_power_state, powerstate)
    lock.release()


def on_message(client, userdata, msg):
    logging.debug(
        "incoming messages: topic:" + msg.topic + " payload:" + str(msg.payload)
    )


client = mqtt.Client(client_id=client_id, protocol=mqtt.MQTTv5)
client.on_connect = on_connect
client.on_message = on_message
client.message_callback_add(localtopic + "/0/switch", on_switch)
client.message_callback_add(localtopic + "/0/voltage_target", on_voltage)
# client.message_callback_add(localtopic + "/update_interval", on_update_interval)
logging.info(f"Connecting to server {args.server}:{args.port}...")
client.connect(args.server, args.port, 60)

retryer = Retrying(stop=stop_after_attempt(10), reraise=True)

model = retryer(powersupply.get_model)
logging.info(f"{model=}")
client.publish(topic=localtopic + "/model", payload=model, qos=2, retain=False)

# client.publish(topic=localtopic + "/update_interval", payload=update_interval, qos=2, retain=False)
client.loop_start()

while True:
    lock.acquire()
    try:
        voltage, current, status = powersupply.get_measurements()
    except IOError as e:
        logging.error(e, exc_info=True)
        continue
    finally:
        lock.release()
    logging.debug(f"posting measurements: {voltage=}, {current=}, {status=}")
    client.publish(
        topic=localtopic + "/0/voltage", payload=voltage, qos=1, retain=False
    )
    client.publish(
        topic=localtopic + "/0/current", payload=current, qos=1, retain=False
    )
    time.sleep(update_interval)
