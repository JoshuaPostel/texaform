from typing import Tuple
from collections import Counter
from time import sleep

from characters import name_char_map
from utils import Point
from agent import Agent
from hud import HUD

min_smelt = 500
max_smelt = 1500


class Smelter(Agent):

    def __init__(self, port: int, location: Point):
        super().__init__(port, location)
        self.powered = False

    def stat(self) -> Tuple[int, Counter, Counter]:
        parts = self.send("STAT").split()
        temp = int(parts[1])
        buffer_in = parts[2]
        if buffer_in == "_":
            buffer_in = ""
        buffer_out = parts[3]
        if buffer_out == "_":
            buffer_out = ""
        return (temp, Counter(list(buffer_in)), Counter(list(buffer_out)))

    def power_on(self):
        if not self.powered:
            self.send("POWR")
            self.powered = not self.powered

    def power_off(self):
        if self.powered:
            self.send("POWR")
            self.powered = not self.powered

    def temp(self) -> int:
        return self.stat()[0]


    def smelt(self, hud: HUD, output: str):
        output_char = name_char_map[output]
        (temp, _, buf_out) = self.stat()
        previous_count = buf_out[output_char]
        while True:
            if hud.accumulator_charge() == 0:
                seconds = 60
                print(f"ACC CHARGE 0, powering off for {seconds} sec")
                self.power_off()
                sleep(seconds)
            (temp, _, buf_out) = self.stat()
            if temp < (min_smelt + 50):
                self.power_on()
            if temp > (max_smelt - 50):
                self.power_off()
            if buf_out[output_char] > previous_count:
                self.power_off()
                break
            # an unluckily timed dog pick could cause this to fail
            previous_count = buf_out[output_char]






