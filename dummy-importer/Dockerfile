FROM python:3.8-slim-buster

ENV ZMQ_PRODUCER_ADDRESS="localhost"
ENV ZMQ_PRODUCER_PORT=5561
RUN apt-get update && apt-get install -y libzmq3-dev

RUN pip install pyzmq

WORKDIR producer
COPY ./producer.py /producer/producer.py

ENTRYPOINT [ "python", "/producer/producer.py" ]
EXPOSE $ZMQ_PRODUCER_PORT
