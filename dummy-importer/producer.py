import time
import zmq
import os

def producer():
    ZMQ_PRODUCER_ADDRESS = os.environ["ZMQ_PRODUCER_ADDRESS"]
    ZMQ_PRODUCER_PORT = os.environ["ZMQ_PRODUCER_PORT"]
    ZMQ_PRODUCER_ADDRESS = f'tcp://{ZMQ_PRODUCER_ADDRESS}:{ZMQ_PRODUCER_PORT}'
    context = zmq.Context()
    zmq_socket = context.socket(zmq.PUSH)
    zmq_socket.connect(ZMQ_PRODUCER_ADDRESS)

    num = 0
    while True:
        work_message = [b"test", bytes(f'num: {num}', encoding='utf-8')]
        zmq_socket.send_multipart(work_message)
        print(f"Sent message: {work_message}")
        time.sleep(1)
        num += 1

producer()
