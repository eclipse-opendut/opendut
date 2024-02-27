import time, random, sys
import can

interface = "socketcan"
channel = "vcan0"

ping_arbid = 0x333
responder_lifetime = 20
response_timeout = 2
n_pings = 5
ping_sleep = 1


def respond_to_pings():
    bus = can.Bus(channel=channel, interface=interface)
    start_time = time.time()
    while time.time() < start_time + responder_lifetime:
        msg = bus.recv(1)
        if msg is None:
            continue
        if msg.arbitration_id == ping_arbid:
            ping_payload = int.from_bytes(msg.data, byteorder="big", signed=False)
            if ping_payload % 2 == 1:
                # for ping requests, (payload % 2 == 0), for ping responses, (payload % 2 == 1)
                continue
            resp_data = ping_payload + 1

            resp = can.Message(
                arbitration_id=ping_arbid,
                data=resp_data.to_bytes(8, byteorder="big"),
                is_extended_id=False,
            )
            bus.send(resp)
    bus.shutdown()


def send_ping(bus):
    ping_payload_int = random.randrange(0, 2**64 - 1, step=2)
    expected_resp_payload = (ping_payload_int + 1).to_bytes(8, byteorder="big")

    msg = can.Message(
        arbitration_id=ping_arbid,
        data=ping_payload_int.to_bytes(8, byteorder="big"),
        is_extended_id=False,
    )
    send_time = time.time()
    bus.send(msg)

    while time.time() < send_time + response_timeout:
        msg = bus.recv(1)
        recv_time = time.time()
        if msg is None:
            continue
        if (
            msg.arbitration_id == ping_arbid
            and bytes(msg.data) == expected_resp_payload
        ):
            latency = int((recv_time - send_time) * 1000)
            return latency

    return None


def run_ping():
    print("Checking whether other peer responds to CAN pings...")
    bus = can.Bus(channel=channel, interface=interface)

    latencies = []
    for _ in range(n_pings):
        if (latency := send_ping(bus)) is not None:
            latencies.append(latency)
        time.sleep(ping_sleep)

    if len(latencies):
        avg = int(sum(latencies) / len(latencies))
        lowest = min(latencies)
        highest = max(latencies)
        loss_percent = int(((n_pings - len(latencies)) / n_pings) * 100)
        unit = " ms"
    else:
        avg, lowest, highest = "n.a.", "n.a.", "n.a."
        loss_percent = 100
        unit = ""

    print(
        f"CAN ping stats (n={n_pings}): avg: {avg}{unit}, lowest: {lowest}{unit}, highest: {highest}{unit}, loss: {loss_percent}%"
    )

    bus.shutdown()


if __name__ == "__main__":
    runtype = sys.argv[1]
    if runtype == "sender":
        run_ping()
    elif runtype == "responder":
        respond_to_pings()
    else:
        print("Invalid argument, must be 'sender' or 'responder'.")
