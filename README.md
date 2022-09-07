# iot-krewetka

## What's that
It is an agent in detection system, which detect threads in a network. There is widely known format of exported data - NetFlow v9.

## This agent is responsible for:
* consuming exported data by collector tool, in our case it is `nProbe`
* transfroming data
* labeling data 
* exporting it to a centralized store (initially it will be kafka, although there is nothing holding us back from implementing other exporters)

## Configuration

Currently configuration is done in yaml file and/or env variables

|parameter|type|description|
|:--|:--:|:--|
|importer.source|enum (zmq)|type of importer|
|importer.settings.zmq_address|string|address of the zmq queue socket. *requires source to be zmq|
|importer.settings.zmq_queue_name|string|name of the queue from where events will be imported. *requires source to be zmq|
|exporter.destination|enum (kafka)|type of exporter|
|exporter.kafka_brokers|list of strings|addresses of kafka brokers. *requires destination to be kafka|
|exporter.kafka_topic|string|kafka topic to which event will be streamed. *requires destination to be kafka|

