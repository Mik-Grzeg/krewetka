dpkg --add-architecture armhf
apt-get update
DEBIAN_FRONTEND=noninteractive apt-get install -yy libzmq3-dev:armhf libssl-dev:armhf wget cmake libprotobuf-dev
wget https://github.com/protocolbuffers/protobuf/releases/download/v21.7/protobuf-all-21.7.tar.gz
tar -xzf protobuf-all-21.7.tar.gz

cd protobuf-21.7
cmake .
cmake --build . --parallel 10
cmake --install .
ldconfig

protoc --version
