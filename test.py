import socket
import time

HOST = '192.168.1.13'  # ESP32-S2のIPアドレスに置き換える
PORT = 8080

while True:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((HOST, PORT))
        s.sendall(b'on')
        data = s.recv(1024)
        print(f"Received: {data.decode()}")

    time.sleep(1)
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((HOST, PORT))
        s.sendall(b'off')
        data = s.recv(1024)
        print(f"Received: {data.decode()}")

    time.sleep(1)

