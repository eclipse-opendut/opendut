#!/usr/bin/env python3

from flask import Flask

app = Flask(__name__)

count = 1


@app.route("/")
def hello_world():
    global count
    count = count + 1
    return f"{count}"


if __name__ == '__main__':
    app.run(host="0.0.0.0", port=5000)
