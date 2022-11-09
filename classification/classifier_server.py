from proto import flow_pb2_grpc
from proto import flow_pb2
import logging
import os
import random
import grpc
from concurrent import futures
import pickle
from sklearn.tree import DecisionTreeClassifier

class FlowMessageClassifierServicer(flow_pb2_grpc.FlowMessageClassifierServicer):

    def __init__(self):
        self.model = Classifier()
        super().__init__()

    def ClassifyStreaming(self, request_iterator, context):
        print('mega printa')
        for msg in request_iterator:
            result = self.model.classify(msg)

            if result[0] == 1:
                result = True
            else:
                result = False

            yield flow_pb2.FlowMessageClass(malicious = result)

    def Classify(self, request, context):
        result = self.model.classify(request)

        if result[0] == 1:
            result = True
        else:
            result = False

        return flow_pb2.FlowMessageClass(malicious = result)


class Classifier:
    model = pickle.load(open('flow-or-malicious.model', 'rb'))

    def classify(self, msg: flow_pb2.FlowMessage):
        return Classifier.model.predict([[
            msg.L4SrcPort,
            msg.L4DstPort,
            msg.Protocol,
            msg.L7Proto,
            msg.InBytes,
            msg.OutBytes,
            msg.InPkts,
            msg.OutPkts,
            msg.TCPFlags,
            msg.FlowDurationMilliseconds
        ]])

def serve(port):
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    flow_pb2_grpc.add_FlowMessageClassifierServicer_to_server(
        FlowMessageClassifierServicer(), server
    )

    server.add_insecure_port(f'[::]:{port}')
    logging.info(f'started listening on: [::]:{port}')

    server.start()
    server.wait_for_termination()

if __name__ == '__main__':
    logging.basicConfig(level=logging.DEBUG)

    port = int(os.environ.get("GRPC_SERVER_PORT", 50051))
    serve(port)
