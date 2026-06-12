import argparse
import random
import sys
import time

import can

# https://python-can.readthedocs.io/en/stable/configuration.html#interface-names
CAN_INTERFACE_BACKEND = "socketcan"
CAN_CHANNEL_NAME = "vcan0"  # the network interface name when socketcan is used

PING_ARBITRATION_ID = 0x333
RESPONDER_LIFETIME_SECONDS = 20
RESPONSE_TIMEOUT_SECONDS = 2
NUMBER_OF_PINGS = 5
PING_SLEEP_DELAY_SECONDS = 1


def run_ping_responder(can_bus):
    start_time = time.time()
    while time.time() < start_time + RESPONDER_LIFETIME_SECONDS:
        msg = can_bus.recv(1)
        if msg is None:
            continue
        if msg.arbitration_id == PING_ARBITRATION_ID:
            ping_payload = int.from_bytes(msg.data, byteorder="big", signed=False)
            if ping_payload % 2 == 1:
                # for ping requests, (payload % 2 == 0), for ping responses, (payload % 2 == 1)
                continue
            resp_data = ping_payload + 1

            resp = can.Message(
                arbitration_id=PING_ARBITRATION_ID,
                data=resp_data.to_bytes(8, byteorder="big"),
                is_extended_id=False,
            )
            can_bus.send(resp)


def send_ping(can_bus):
    ping_payload_int = random.randrange(0, 2**64 - 1, step=2)
    expected_resp_payload = (ping_payload_int + 1).to_bytes(8, byteorder="big")

    msg = can.Message(
        arbitration_id=PING_ARBITRATION_ID,
        data=ping_payload_int.to_bytes(8, byteorder="big"),
        is_extended_id=False,
    )
    send_time = time.time()
    can_bus.send(msg)

    while time.time() < send_time + RESPONSE_TIMEOUT_SECONDS:
        msg = can_bus.recv(1)
        recv_time = time.time()
        if msg is None:
            continue
        if (
            msg.arbitration_id == PING_ARBITRATION_ID
            and bytes(msg.data) == expected_resp_payload
        ):
            latency = int((recv_time - send_time) * 1000)
            return latency

    return None


def run_ping_sender(can_bus):
    print("Checking whether other peer responds to CAN pings...")
    latencies = []

    for _ in range(NUMBER_OF_PINGS):
        if (latency := send_ping(can_bus)) is not None:
            latencies.append(latency)
        time.sleep(PING_SLEEP_DELAY_SECONDS)

    if latencies:
        avg = int(sum(latencies) / len(latencies))
        lowest = min(latencies)
        highest = max(latencies)
        loss_percent = int(((NUMBER_OF_PINGS - len(latencies)) / NUMBER_OF_PINGS) * 100)
        unit = " ms"
    else:
        avg, lowest, highest = "n.a.", "n.a.", "n.a."
        loss_percent = 100
        unit = ""

    print(
        f"CAN ping stats (n={NUMBER_OF_PINGS}): avg: {avg}{unit}, lowest: {lowest}{unit}, highest: {highest}{unit}, loss: {loss_percent}%"
    )

    # Make this script exit with a code indicating error if not all ping messages went through
    if loss_percent > 0:
        print("Error: CAN messages were lost during ping test.")
        return False

    return True


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="CAN ping test utility")
    parser.add_argument("runtype", choices=["sender", "responder"], help="Run as 'sender' or 'responder'")
    args = parser.parse_args()

    with can.Bus(channel=CAN_CHANNEL_NAME, interface=CAN_INTERFACE_BACKEND) as can_bus:
        if args.runtype == "sender":
            if not run_ping_sender(can_bus):
                sys.exit(1)
        elif args.runtype == "responder":
            run_ping_responder(can_bus)
