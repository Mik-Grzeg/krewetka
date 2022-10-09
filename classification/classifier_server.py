from proto import flow_pb2_grpc
from proto import flow_pb2
import random
import grpc
from concurrent import futures
import pickle
from sklearn.tree import DecisionTreeClassifier

class FlowMessageClassifierServicer(flow_pb2_grpc.FlowMessageClassifierServicer):

    def ClassifyStreaming(self, request_iterator, context):
        print('mega printa')
        for msg in request_iterator:
            result = Classifier.classify(msg)

            if result[0] == 1:
                result = True 
            else:
                result = False
            
            yield flow_pb2.FlowMessageClass(malicious = result)
 

class Classifier:
    model = pickle.load(open('flow-or-malicious.model', 'rb'))

    @classmethod
    def classify(__cls__, msg: flow_pb2.FlowMessage):
        return __cls__.model.predict([[
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

def serve():
    print('serving')
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    flow_pb2_grpc.add_FlowMessageClassifierServicer_to_server(
        FlowMessageClassifierServicer(), server
    )

    server.add_insecure_port('[::]:50051')
    server.start()

    print('started listening on: [::]:50051')
    server.wait_for_termination()

if __name__ == '__main__':

    serve()