import socket
import time

from utils import Point

HOST = "127.0.0.1"  # Standard loopback interface address (localhost)
freq = 0.02
#freq = 1


class Agent:

    # TODO post_init() method to clean up child classes
    def __init__(self, port: int, location: Point):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.connect((HOST, port))
        self.location: Point = location

    def send(self, msg: str) -> str:
        self.socket.sendall(str.encode(msg + ";"))
        # TODO move to logging setup
        #print(f"sent: {msg}")
        buffer = b""
        while b";" not in buffer:
            data = self.socket.recv(32)
            buffer += data
        response = buffer.decode("utf-8")
        # strip the trailing ;
        response = response[:-1]
        # TODO move to logging setup
        #print(f"recv: {response}")
        return response

    def kind(self) -> str:
        response = self.send("KIND")
        name = response.split()[1]
        return name


