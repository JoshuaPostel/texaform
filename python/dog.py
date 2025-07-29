import time
import random
from typing import Tuple, Dict, Set, Optional, Self
from collections import defaultdict

from characters import name_char_map
from utils import Point, Rect, Direction
from agent import Agent



class Dog(Agent):

    def __init__(self, port: int):
        # fake location to start
        super().__init__(port, Point(-1, -1))
        self.char_locations: Dict[str, Set[Point]] = defaultdict(set)
        self.goto_recursion_count = 0
        (location, _, _, _) = self.stat()
        self.location: Point = location
        self.facing: Direction = Direction.E
        # TODO add these to init arguments
        self.charge_location: Optional[Point] = None
        self.depot_area: Rect = Rect(Point(0,0), Point(0,0))

    def charge_if_needed(self):
           (_, _, charge, _) = self.stat()
           if charge < 90:
               if not self.charge_location:
                   raise ValueError("DID NOT INIT CHARGE LOCATION")
               else:
                   print("attempting to charge")
                   self.approach(self.charge_location)
                   self.send("CHRG")
                   seconds = (100 - charge) / 10 
                   print(f"sleeping for {seconds} seconds")
                   time.sleep(seconds)


    def send(self, msg: str) -> str:
        resp = super().send(msg)
        if resp == "ERRR low battery":
            seconds = 200
            print(f"sleeping for {seconds} to let dog charge")
            time.sleep(seconds)
            (pos, facing, _, _) = self.stat()
            self.charge_if_needed()
            print("returning to position")
            self.goto(pos)
            self.face(facing)
            print("resuming")
            return self.send(msg)

        return resp

    def pick(self, char: str, target: Optional[Point] = None):
        self.send(f"PICK {char}")
        if target is None:
            target = self.location.forward(self.facing)
#           (pos, facing, _, _) = self.stat()
#           target = pos.forward(facing)
        try:
            self.char_locations[char].remove(target)
        except KeyError:
            print(f"{char} {target} was not in known locations")

    def stat(self) -> Tuple[Point, str, int, str]:
        reply = self.send("STAT")
        parts = reply.split(" ")
        cx = int(parts[1])
        cy = int(parts[2])
        facing = parts[3]
        battery = int(parts[4].strip("%"))
        payload = parts[5]
        self.location = Point(cx, cy)
        self.facing = Direction(facing)
        return (self.location, facing, battery, payload)


    def scan(self) -> Tuple[str, str, str]:
        reply = self.send("SCAN")
        (point, facing, _, _) = self.stat()
        chars = (reply[5], reply[6], reply[7])
        for i, char in enumerate(chars):
            char_location = point.forward(facing, i + 1)
            if not self.depot_area.contains(char_location):
                self.char_locations[char].add(char_location)

        return chars

    def turn_right(self):
        self.send("TURN R")
        self.facing = self.facing.right()

    def turn_left(self):
        self.send("TURN L")
        self.facing = self.facing.left()

    def face(self, direction: Direction):
        if self.facing == direction:
            return
        else:
            self.turn_right()
            return self.face(direction)

    def move(self):
        self.send("MOVE")
        self.location = self.location.forward(self.facing)

# TODO integrate this with goto
#    def go_through(self):
#        inital_direction = self.facing
#        (p1, _, _) = self.scan()
#        if p1 == ".":
#            self.move()
#        if p1 in ["I", "O", "L", "U"]:
#            self.pick(p1)
#            c = ""
#            while c != ".":
#                self.turn_left()
#                (c, _, _) = self.scan()
#            self.send("DROP")
#            self.face(inital_direction)


    def hug_right(self, desired_direction: Direction):
        (p1, _, _) = self.scan()
        if p1 == ".":
            self.move()
            if self.facing == desired_direction:
                return
            else:
                self.turn_left()
                return self.hug_right(desired_direction)
        else:
            self.turn_right()
            return self.hug_right(desired_direction)

    def occupied_locations(self) -> Set[Point]:
        locations = set()
        for char in self.char_locations.keys():
            if char != ".":
                for pos in self.char_locations[char]:
                    locations.add(pos)
        return locations


    def goto(self, target: Point, skip_x: bool = False) -> bool:
        self.goto_recursion_count += 1
        #print(f"GOTO command count: {self.goto_recursion_count}")
        if self.goto_recursion_count > 100:
            self.goto_recursion_count = 0
            print(f"failed to goto {target} in under 100 commands")
            return False
        if target in self.occupied_locations():
            print(f"failed to goto {target}, location occupied")
            return False
        if self.location.x == target.x and self.location.y == target.y:
            self.goto_recursion_count = 0
            print(f"reached destination: {target}")
            return True
        if self.location.x < target.x and not skip_x:
            self.face("E")
            self.hug_right("E")
            return self.goto(target, False)
        if target.x < self.location.x and not skip_x:
            self.face("W")
            self.hug_right("W")
            return self.goto(target, False)
        if self.location.y < target.y:
            self.face("S")
            self.hug_right("S")
            return self.goto(target, True)
        if target.y < self.location.y:
            self.face("N")
            self.hug_right("N")
            return self.goto(target, True)
        return self.goto(target, False)

    def _generate_random_point_within_40_of_charge(self) -> Point:
        x_delta = random.randint(-10, 10)
        y_delta = random.randint(-10, 10)
        x = max(self.location.x + x_delta, 0)
        y = max(self.location.y + y_delta, 0)
        point = Point(x, y)
        distance = point.taxi_distance(self.charge_location)
        if distance < 40:
            return point
        else:
            return self._generate_random_point_within_40_of_charge()


    def _generate_random_point(self, min_dist: Optional[int] = None, max_dist: int = 100) -> Point:
        x_delta = random.randint(-10, 10)
        y_delta = random.randint(-10, 10)
        x = max(self.location.x + x_delta, 0)
        y = max(self.location.y + y_delta, 0)
        point = Point(x, y)
        distance = point.taxi_distance(self.location)
        if min_dist and distance < min_dist:
            return self._generate_random_point(min_dist, max_dist)
        if max_dist and max_dist < distance:
            return self._generate_random_point(min_dist, max_dist)
        else:
            return point

    def find_char_random_walk(self, char: str):
        if self.char_locations.get(char):
            return
        else:
            target = self._generate_random_point_within_40_of_charge()
            print(f"random walk destination: {target}")
            self.goto(target)
            self.find_char_random_walk(char)

    def approach(self, point: Point):
        north = point.forward("N")
        if north in self.char_locations["."]:
            self.goto(north)
            self.face("S")
            return
        south = point.forward("S")
        if south in self.char_locations["."]:
            self.goto(south)
            self.face("N")
            return
        east = point.forward("E")
        if east in self.char_locations["."]:
            self.goto(east)
            self.face("W")
            return
        west = point.forward("W")
        if west in self.char_locations["."]:
            self.goto(west)
            self.face("E")
            return
        # TODO handle case where 
        # * point is surrounded 
        # * not in dogs memory 
        success = self.goto(west)
        if success:
           self.face("E")
        else:
            # TODO this can still fail
            rand = self._generate_random_point(max_dist=4)
            self.goto(rand)
            self.approach(point)


    def pick_known(self, char: str, point: Point):
        print(f"attempting to fetch {char} at {point}")
        self.approach(point)
        self.pick(char, point)


    def drop_off(self, point: Point):
        self.approach(point)
        self.send("DROP")

    def drop_within(self, area: Rect):
        target = area.random_point_inside()
        self.approach(target)
        (p1, _, _) = self.scan()
        if p1 == ".":
            self.send("DROP")
            return
        else:
            self.drop_within(area)


    def transport(self, char: str, from_loc: Point, to_loc: Point):
        self.pick_known(char, from_loc)
        self.drop_off(to_loc)


    def pick_unknown(self, char: str):
        if len(char) > 1:
            char = name_char_map[char]
        known_locations = self.char_locations.get(char)
        if known_locations:
            print(f"known location of {char}: fetching")
            location = list(known_locations)[0]
            self.pick_known(char, location)
            (_, _, _, payload) = self.stat()
            if payload == char:
                print(f"successfully fetched {char}")
                return
            else:
                print(f"failed to fetch {char} trying again")
                self.pick_unknown(char)

        else:
            print(f"no known locations of {char}: searching")
            self.find_char_random_walk(char)
            self.pick_unknown(char)

    def fetch(self, name: str, drop_location: Point):
        fetch_char = name_char_map[name]
        print(f"FETCHING {fetch_char}")
        self.charge_if_needed()
        self.pick_unknown(fetch_char)
        self.drop_off(drop_location)

    def build_at(self, location: Point) -> Optional[int]:
        west = location.forward("W")
        self.goto(west)
        self.face("E")
        resp = self.send("BULD")
        parts = resp.split(" ")
        try:
            return int(parts[1])
        except Exception:
            return None







        
