FROM ubuntu:18.04

RUN apt update && apt install -y software-properties-common

RUN add-apt-repository -y ppa:luke-jr/bitcoincore && \
  apt update && \
  apt install -y \
  curl \
  wget \
  tar \
  python \
  build-essential \
  libzmq3-dev \
  libsnappy-dev && \
  apt install -y \
  bitcoind && \
  curl --silent --location https://deb.nodesource.com/setup_10.x | bash -

RUN apt install -y \
  git \
  vim \
  nodejs && \
  mkdir /root/regtest-data && \
  echo "satoshi" > /root/regtest-data/KEYS

WORKDIR /root

COPY . .
RUN mv docker/* .

RUN chmod +x install_leveldb.sh && \
  chmod +x run_bitcoind_service.sh && \
  chmod +x run_regtest_app.sh && \
  chmod +x create_bitcoin_wallet.sh && \
  ./install_leveldb.sh

# Change the checkout branch if you need to. Must fetch because of Docker cache
# RUN git fetch origin && \
#   git checkout ebee446d7c3b9071633764b39cdca3ac1b28d253

RUN npm i

ENTRYPOINT ["/root/run_regtest_app.sh"]

EXPOSE 8080
