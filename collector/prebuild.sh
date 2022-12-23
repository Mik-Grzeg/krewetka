dpkg --add-architecture armhf
apt-get update
DEBIAN_FRONTEND=noninteractive apt-get install -yy libzmq3-dev:armhf libssl-dev:armhf libprotobuf-dev protobuf-compiler libsasl2-dev zlib1g:armhf zlib1g-dev:armhf
