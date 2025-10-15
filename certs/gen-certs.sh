#!/bin/bash -eu

cd $(dirname $0)

[ -f server.key ] && exit 0

# CN
COMMON_NAME=${COMMON_NAME:-"localhost"}

CDIR=".certs"
rm -rf "$CDIR"
mkdir "$CDIR"

# CA
CA_KEY=${CA_KEY:-"$CDIR/ca.key"}
CA_CRT=${CA_CRT:-"$CDIR/ca.crt"}
# server
SERVER_KEY=${SERVER_KEY:-"$CDIR/server.key"}
SERVER_CSR=${SERVER_CSR:-"$CDIR/server.csr"}
SERVER_CRT=${SERVER_CRT:-"$CDIR/server.crt"}
SERVER_SSV2_CERT=${SERVER_SSV2_CERT:-"$CDIR/server-cert-ssv2.json"}
# client
CLIENT_KEY=${CLIENT_KEY:-"$CDIR/client.key"}
CLIENT_CSR=${CLIENT_CSR:-"$CDIR/client.csr"}
CLIENT_CRT=${CLIENT_CRT:-"$CDIR/client.crt"}

echo "Generating the CA Key and Certificate"
openssl req -x509 -sha256 -newkey rsa:2048 -keyout "${CA_KEY}" -out "${CA_CRT}" -days 356 -nodes -subj "/CN=$(whoami)'s Cert Authority"

echo "Generating the Server Key and Certificate"
openssl req -newkey rsa:2048 -keyout "${SERVER_KEY}" -out "${SERVER_CSR}" -sha256 -nodes -subj "/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=${COMMON_NAME}"

echo "Signing the server certificate with the CA Certificate"
openssl x509 -req -extfile <(printf "subjectAltName=DNS:${COMMON_NAME},IP:127.0.0.1") -sha256 -days 365 -in "${SERVER_CSR}" -CA "${CA_CRT}" -CAkey "${CA_KEY}" -CAcreateserial -out "${SERVER_CRT}"

echo "Generating the Client Key and Certificate"

openssl req -newkey rsa:2048 -keyout "${CLIENT_KEY}" -out "${CLIENT_CSR}" -sha256 -nodes -subj "/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=${COMMON_NAME}"

echo "Signing the client certificate with the CA Certificate"
openssl x509 -req -extfile <(printf "subjectAltName=DNS:${COMMON_NAME},IP:127.0.0.1") -sha256 -days 365 -in "${CLIENT_CSR}" -CA "${CA_CRT}" -CAkey "${CA_KEY}" -CAcreateserial -out "${CLIENT_CRT}"

cp -av $SERVER_KEY $SERVER_CRT $CA_CRT ./
