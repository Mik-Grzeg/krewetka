import grpc
from proto import flow_pb2
from proto import flow_pb2_grpc

def generate_messages():
    messages = [
        flow_pb2.FlowMessage(
            OutBytes = 2,
            OutPkts = 1,
            InBytes = 1,
            InPkts = 2,
            IPV4SrcAddr = '192.168.1.1',
            IPV4DstAddr = '192.168.1.2',
            L7Proto = '17.0',
            L4DstPort = 5051,
            L4SrcPort = 4079,
            FlowDurationMilliseconds = 51,
            Protocol = 1,
            TCPFlags = 2
        )
    ]

    for msg in messages:
        print(f'Sending message: {msg}')
        yield msg

def run_classify(stub):
    responses = stub.ClassifyStreaming(generate_messages())
    for response in responses:
        print(f'received message: {response.malicious}')


def run():
    with grpc.insecure_channel('localhost:50051') as channel:
        stub = flow_pb2_grpc.FlowMessageClassifierStub(channel)
        run_classify(stub)

if __name__ == '__main__':
    run()