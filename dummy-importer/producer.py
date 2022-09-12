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

    num = 0
    while True:
        print(f"n = {num}")
        work_message = [b"test", bytes(f'test num: {num}', encoding='utf-8')]
        zmq_socket.send_multipart(work_message)
        print(f"Sent message: {work_message}")
        time.sleep(random.uniform(0,1))
        num += 1

if __name__ == '__main__':
    producer()
