# collector
Application which can be ran on a device like raspberry pi. it relies on external tool called `nprobe`, to be more specific, on the format that the tool exports collected packets in. It relies on data streaming platform **Kafka**.

Collector is written in Rust and cross compiled to the arm64 architecture with [`cross`](https://github.com/cross-rs/cross)
## What is it responsible for
* consuming exported data by external tool, which is `nProbe`
* transfroming data
* labeling data
* exporting the data to a centralized processing service via kafka

## Configuration

### nProbe
nProbe has to be exporting data to ZMQ queue in specific format, hence there is specific command to start nProbe with

```bash
nprobe -i eth0 -V 9 -n none -T %OUT_BYTES%OUT_PKTS%L4_DST_PORT%IPV4_DST_ADDR%IPV4_SRC_ADDR%PROTOCOL%L4_SRC_PORT%IN_BYTES%IN_PKTS%L7_PROTO%TCP_FLAGS%FLOW_DURATION_MILLISECONDS --zmq tcp://localhost:5561 --json-labels --zmq-disable-compression --zmq-format j
```

### collector
Currently configuration is done in yaml file named `krewetka.yaml` or env variables. The configuration file has to be present in a current directory.

|parameter|type|description|
|:--|:--:|:--|
|importer.source|enum (zmq)|type of importer|
|importer.settings.zmq_address|string|address of the zmq queue socket. *requires source to be zmq|
|importer.settings.zmq_queue_name|string|name of the queue from where events will be imported. *requires source to be zmq|
|exporter.destination|enum (kafka)|type of exporter|
|exporter.kafka_brokers|string|addresses of kafka brokers in kafka format - `broker1:9092,broker2:9092` *requires destination to be kafka|
|exporter.kafka_topic|string|kafka topic to which event will be streamed. *requires destination to be kafka|


Examplar configuration looks like this
```yaml
# krewetka.yaml
---

importer:
  source: zmq
  settings:
    zmqAddress: 127.0.0.1:5561
    zmqQueueName: flow
exporter:
  destination: kafka
  settings:
    kafkaBrokers: "krewetka.norwayeast.cloudapp.azure.com:9094"
    kafkaTopic: flows
```

