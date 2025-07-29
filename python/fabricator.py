from collections import Counter

from characters import name_char_map
from agent import Agent

class Fabricator(Agent):

#    def build_agent(self, cls, pos: Point) -> Any:
#        name = cls.__name__.upper()
#        if name.lower() == "lazercutter":
#            name = "LAZER_CUTTER"
#        recv = self.send(f"BULD {name} {pos.x} {pos.y}")
#        port = int(recv.split()[1])
    def _buffer(self, idx: int) -> Counter:
        recv = self.send("STAT")
        buffer = recv.split()[idx]
        if buffer == "_":
            buffer = ""
        return Counter(list(buffer))

    def buffer_in(self) -> Counter:
        return self._buffer(1)

    def buffer_out(self) -> Counter:
        return self._buffer(2)

    def resource_count(self, name: str) -> int:
        char = name_char_map[name]
        count = self.buffer_in().get(char)
        return count or 0

    def research(self):
        self.send("RESR")
