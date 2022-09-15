import json
import time
import zmq
import os
import random


def producer():

    ZMQ_PRODUCER_ADDRESS = os.environ["ZMQ_PRODUCER_ADDRESS"]
    ZMQ_PRODUCER_PORT = os.environ["ZMQ_PRODUCER_PORT"]
    ZMQ_PRODUCER_ADDRESS = f'tcp://*:{ZMQ_PRODUCER_PORT}'
    print(f"{ZMQ_PRODUCER_ADDRESS}")
    context = zmq.Context()
    zmq_socket = context.socket(zmq.PUB)
    zmq_socket.bind(ZMQ_PRODUCER_ADDRESS)


    while True:
        msg = dict(
            OUT_BYTES = random.randint(30, 1000),
            OUT_PKTS = random.randint(0, 10),
            L4_DST_PORT = random.randint(1, 33333),
            IPV4_DST_ADDR = ".".join(map(str, (random.randint(0, 255) for _ in range(4)))),
            IPV4_SRC_ADDR = ".".join(map(str, (random.randint(0, 255) for _ in range(4)))),
            PROTOCOL =  random.randint(0, 20),
            IN_BYTES = random.randint(50, 500),
            IN_PKTS = random.randint(0, 10),
            L7_PROTO = "Unknown",
            TCP_FLAGS = random.randint(0, 20),
            FLOW_DURATION_MILLISECONDS = random.randint(0, 100000)
        )

        print(f"{msg}")
        work_message = [b"flows", bytes(f'flow {json.dumps(msg)}', encoding='utf-8')]
        zmq_socket.send_multipart(work_message)
        print(f"Sent message: {work_message}")
        time.sleep(random.uniform(0,2))
if __name__ == '__main__':
    producer()
