from gltesting.fixtures import *
import time
import shutil
import os
import socket

def test_regtest(scheduler, nobody_id, root_id, bitcoind, node_factory):
    """Create the scheduler and write his connection details in local files.
    """

    os.mkdir("regtest")

    l1 = node_factory.get_node()
    sock = socket.socket()
    sock.bind(('', 0))
    _, free_port = sock.getsockname()
    print(str(free_port));
    del sock
    l1_dir = l1.info['lightning-dir']

    write_file("regtest/gl_l1_port.txt", str(free_port))
    write_file("regtest/gl_bitcoind_rpcport.txt", str(bitcoind.rpcport))
    write_file("regtest/gl_nobody_key.txt", nobody_id.private_key.decode("utf-8"))
    write_file("regtest/gl_cert_chain.txt", nobody_id.cert_chain.decode("utf-8"))
    write_file("regtest/gl_grpc_addr.txt", scheduler.grpc_addr)
    write_file("regtest/gl_ca_crt.txt", root_id.cert_chain.decode("utf-8"))

    time.sleep(1)  # otherwise lightning node may have not created the unix socket

    os.system('socat "TCP4-listen:{},fork,reuseaddr" "UNIX-CONNECT:{}/lightning-rpc" &'.format(free_port, l1_dir))

    try:
        time.sleep(100000)
    finally:
        print("Deleting regtest dir")
        shutil.rmtree('regtest')
        l1.rpc.stop()


def write_file(path, content):
    f = open(path, "w")
    f.write(content)
    f.close()
