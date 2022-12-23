dpkg --add-architecture armhf
apt-get update
DEBIAN_FRONTEND=noninteractive apt-get install -yy libzmq3-dev:armhf libssl-dev:armhf libprotobuf-dev protobuf-compiler zlib1g zlib1g-dev
