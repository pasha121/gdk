BASE=${PWD}/gdk_greenlight/regtest
export GL_NOBODY_CRT="${BASE}/gl_cert_chain.txt"
export GL_NOBODY_KEY="${BASE}/gl_nobody_key.txt"
export GL_CA_CRT="${BASE}/gl_ca_crt.txt"
export GL_SCHEDULER_GRPC_URI=`cat ${BASE}/gl_grpc_addr.txt`
export GL_GRPC_URI=`cat ${BASE}/gl_grpc_addr.txt`
export GL_L1_PORT=`cat ${BASE}/gl_l1_port.txt`
export GL_BITCOIND_RPCPORT=`cat ${BASE}/gl_bitcoind_rpcport.txt`

socat "UNIX-listen:/tmp/unix-sock-${GL_L1_PORT},fork,reuseaddr" "TCP4-CONNECT:127.0.0.1:${GL_L1_PORT}" &