# classifier
gRPC server which classifies flow messages based on the input data to be `malicious` or `non-malicious`. Previously trained model is pickled as a `flow-or-malicious.model`. Server import it and the classification is done based on that.


## Configuration

Requires environment variable to specify which port should the server be running on.

```bash
GRPC_SERVER_PORT: <server-port>
```
