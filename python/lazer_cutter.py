from typing import Any, List, Optional, Set, Self, Literal
from collections import Counter
from dataclasses import dataclass

from utils import Point, Shape, ShapeProperties, GEAR
from agent import Agent


PLATE_WIDTH = 24
PLATE_HEIGHT = 12


TEST_SHAPE = """xx.
x..
..."""


class LazerCutter(Agent):

    def __init__(self, port: int, location: Point):
        super().__init__(port, location)
        self.powered = False
        self.cutter_position: Point = Point(0, 0)

#        self.plate
#        # using x left-right, y top-bottom cordinates
#        self.plate: List[List[bool]] = []
#        for x in range(PLATE_WIDTH):
#            self.plate.append([False for y in range(PLATE_HEIGHT)])
#
#        print(f"plate: {self.plate}")

        self.desired_shape: Optional[Shape] 
        self.remaining_to_cut: Optional[Shape]
                

    def power_on(self):
        if not self.powered:
            self.send("POWR")
            self.powered = not self.powered

    def power_off(self):
        if self.powered:
            self.send("POWR")
            self.powered = not self.powered

    def goto(self, target: Point):
        print(f"LC: goto {target}")
        while self.cutter_position != target:
            if target.x > self.cutter_position.x:
                self.send("MVXP")
                self.cutter_position += Point(1, 0)
            elif target.x < self.cutter_position.x:
                self.send("MVXN")
                self.cutter_position += Point(-1, 0)
            elif target.y > self.cutter_position.y:
                self.send("MVYP")
                self.cutter_position += Point(0, 1)
            elif target.y < self.cutter_position.y:
                self.send("MVYN")
                self.cutter_position += Point(0, -1)

        print(f"LC: goto {target}")

    def closest_uncut(self) -> Optional[Point]:
        closest_point = None
        closest_distance = 1000
        for point in self.remaining_to_cut.points:
            distance = self.cutter_position.taxi_distance(point)
            if distance < closest_distance:
                closest_distance = distance
                closest_point = point
        return closest_point
        

    # not packing shapes for now
    def cut_shape(self, props: ShapeProperties):
        self.power_off()
        self.desired_shape = props.shape
        self.remaining_to_cut = self.desired_shape.outline()
        print(self.remaining_to_cut.points)

        # TODO check if material in buffer in
        self.send(f"LOAD {props.material}")
        while len(self.remaining_to_cut.points) > 0:
            self.remaining_to_cut
            target = self.closest_uncut()
            if self.cutter_position.taxi_distance(target) > 1:
                self.power_off()
            self.goto(target)
            self.power_on()
            self.remaining_to_cut.points.remove(target)

        self.power_off()
        pick_location = next(iter(self.desired_shape.points))
        self.goto(pick_location)
        self.send(f"PICK {props.name}")


if __name__ == "__main__":
    s = Shape(TEST_SHAPE)
    print(s.points)
    print(s.outline().points)
