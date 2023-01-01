FROM python:3.8-slim-buster

WORKDIR /classifier

RUN apt-get update && \
    apt-get install -yy protobuf-compiler

COPY requirements.txt .
RUN pip3 install -r requirements.txt && mkdir proto

COPY classifier_server.py flow-or-malicious.model ./
COPY flow.proto  ./proto

RUN  python3 -m grpc_tools.protoc -I=. --python_out=. --grpc_python_out=. ./proto/flow.proto

EXPOSE 50051
CMD [ "python3", "classifier_server.py" ]
