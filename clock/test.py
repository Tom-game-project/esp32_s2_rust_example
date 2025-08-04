import socket
import time

HOST = '192.168.1.13'  # ESP32-S2のIPアドレスに置き換える
PORT = 8080


def send_cmd(cmd:bytes):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((HOST, PORT))
        s.sendall(cmd)
        data = s.recv(1024)
        print(f"Received: {data.decode()}")


while True:
    send_cmd(b'red')
    time.sleep(1)
    send_cmd(b'green')
    time.sleep(1)
    send_cmd(b'blue')
    time.sleep(1)
    send_cmd(b'neopixel_off')
    time.sleep(1)
