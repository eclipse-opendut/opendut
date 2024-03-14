#!/usr/bin/python
import paho.mqtt.client as mqtt
import argparse
import time
import logging
import psutil

logging.getLogger().setLevel("DEBUG")
parser = argparse.ArgumentParser(
    prog="Node stats MQTT-Daemon", description="publishes node statistics"
)
parser.add_argument("--server")
parser.add_argument("--port", type=int, default=1884)
parser.add_argument("--topic", default="defaultpeer/nodestats")
parser.add_argument("--interval", type=float, default=1)
# TODO
parser.add_argument("--device", default="first")
args = parser.parse_args()

# setup args
# Generate a unique ID, that is however persistent between restarts.
client_id = "nodestats_" + args.topic.replace("/", "_")
update_interval = args.interval
localtopic = args.topic


def on_connect(client, userdata, flags, rc, unsure):
    logging.info("Connected with result code " + str(rc))


client = mqtt.Client(client_id=client_id, protocol=mqtt.MQTTv5)
client.on_connect = on_connect
logging.info(f"Connecting to server {args.server}:{args.port}...")
client.connect(args.server, args.port, 60)

# logging.info(f"{model=}")
# client.publish(topic=localtopic + "/model", payload=model, qos=2, retain=False)

# client.publish(topic=localtopic + "/update_interval", payload=update_interval, qos=2, retain=False)
client.loop_start()
oldtime = time.time()
oldcounters = newcounters = psutil.net_io_counters(pernic=True)
while True:
    client.publish(
        topic=localtopic + "/cpu%", payload=psutil.cpu_percent(), qos=1, retain=False
    )
    client.publish(
        topic=localtopic + "/mem%",
        payload=psutil.virtual_memory().percent,
        qos=1,
        retain=False,
    )

    # the net stats are really a lot, and maybe not too interesting, so we send them only at max each 10 seconds.
    newtime = time.time()
    if newtime - oldtime >= 10:
        throughput = dict()
        # todo depends on what stats we are interested in, we should filter it down:
        newcounters = psutil.net_io_counters(pernic=True)
        for ifname, stat in newcounters.items():
            throughput[ifname] = (stat.bytes_sent - oldcounters[ifname].bytes_sent) / (
                newtime - oldtime
            ), (stat.bytes_recv - oldcounters[ifname].bytes_recv) / (newtime - oldtime)
        # print(throughput)
        for ifname, value in throughput.items():
            client.publish(
                topic=f"{localtopic}/net/{ifname}/sent",
                payload=round(value[0], 1),
                qos=1,
                retain=False,
            )
            client.publish(
                topic=f"{localtopic}/net/{ifname}/recv",
                payload=round(value[1], 1),
                qos=1,
                retain=False,
            )

        oldcounters = newcounters
        oldtime = newtime

    time.sleep(update_interval)
