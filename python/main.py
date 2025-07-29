from utils import Point
from make import build, research, make, hud
from dog import Dog
from smelter import Smelter
from lazer_cutter import LazerCutter
from fabricator import Fabricator


# inital dog and fab always exists
fab0 = Fabricator(3334, Point(125, 125))
dog0 = Dog(3335)
dog0.charge_location = Point(130, 127)
dog0.scan()

existing_agents = hud.get_existing_agents()
print(existing_agents)

# stage 1
# research and build smelter
research("SMELTER", dog0, None, None, fab0)
smelters = existing_agents["SMELTER"]
smelter0_location = Point(130, 130)
if smelters:
    smelter0 = Smelter(smelters[0], smelter0_location)
else:
    # we start with the nessisary materials
    port = build("SMELTER", smelter0_location, dog0, None, None, fab0)
    smelter0 = Smelter(port, smelter0_location)

# stage 2
# * research solar, battery, lazer_cutter
research("LAZER_CUTTER", dog0, smelter0, None, fab0)
research("BATTERY", dog0, smelter0, None, fab0)
research("SOLAR", dog0, smelter0, None, fab0)

# stage 3
# * build lazer cutter
lazer_cutters = existing_agents["LAZER_CUTTER"]
lc0_location = Point(123, 130)
if lazer_cutters:
    lc0 = LazerCutter(lazer_cutters[0], lc0_location)
else:
    port = build("LAZER_CUTTER", lc0_location, dog0, smelter0, None, fab0)
    lc0 = LazerCutter(port, lc0_location)

# stage 4
# * research dog
research("DOG", dog0, smelter0, lc0, fab0)

# stage 5
# * research accumulator
# * build accumulator
# * build solar
research("ACCUMULATOR", dog0, smelter0, lc0, fab0)
build("ACCUMULATOR", Point(133, 127), dog0, smelter0, lc0, fab0)
build("SOLAR_PANNEL", Point(135, 125), dog0, smelter0, lc0, fab0)


# stage 6
# * build dogs
# * research victory
research("ACCUMULATOR", dog0, smelter0, lc0, fab0)
if "VICTORY" not in hud.completed_research():
    hud.send("RESR VICTORY")
    for i in range(10):
        print(f"BUILDING DOG {i}")
        make("DOG", dog0, smelter0, lc0, fab0)
        dog0.transport("DOG", fab0.location, fab0.location)
    print("victory eminent")
    assert 2 == 4
    fab0.research()
    fab0.research()

