# reporting
It is an application, which works as a server for generating reports. It relies on database **Clickhouse**. It serves few http and websocket endpoints.

Reporting API is written in **Rust**.

## What is it responsible for
* aggregating data with specific clickhouse queries (read only)
* serializing data to json format
* serving http and websocket endpoint, so clients would be able to receive reporting data 

### Endpoints
* Http - ratios of messages classified as `malicous` and `non-malicious`
* WebSocket 
    *  throughput statistics aggregated by specific interval of time
    * actual data with the details

## Configuration 
Requires exporting environment variables with following names

```bash
KREWETKA__CLICKHOUSE_SETTINGS__HOST: <clickhouse-host>
KREWETKA__CLICKHOUSE_SETTINGS__PORT: <clickhouse-port>
KREWETKA__CLICKHOUSE_SETTINGS__USER: <clickhouse-user>
KREWETKA__CLICKHOUSE_SETTINGS__PASSWORD: <clickhouse-user-password> 
```