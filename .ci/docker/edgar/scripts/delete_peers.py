#!/usr/bin/env python3

"""
Tired of cleaning up the netbird clients?

Just delete all peers belonging to a certain group
"""
import os
import sys

import requests

NETBIRD_API = os.environ.get("NETBIRD_MANAGEMENT_API", default="https://netbird-api.opendut.local")
PEER_LIST = os.environ.get("PEER_URI", default="{}/api/peers".format(NETBIRD_API))
API_TOKEN = os.environ["NETBIRD_API_TOKEN"]
NETBIRD_GROUP = os.environ.get("NETBIRD_GROUP", default="docker")


def delete_all_hosts_in_netbird_group(group_name: str):
    auth_header = {"Authorization": "Token {}".format(API_TOKEN)}
    response = requests.get(PEER_LIST, headers=auth_header)

    delete_list = []
    for peer in response.json():
        result = any(group["name"] == group_name for group in peer["groups"])
        if result:
            print(peer["id"])
            delete_list.append(peer["id"])

    print(f"Deleting {len(delete_list)} peers.")
    for peer in delete_list:
        url = f"{PEER_LIST}/{peer}"
        print(f"Deleting peer id '{peer}' in group '{group_name}'.")
        requests.delete(url, headers=auth_header)


if __name__ == '__main__':
    delete_all_hosts_in_netbird_group(NETBIRD_GROUP)
