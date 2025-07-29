from collections import Counter
from typing import Dict, List
from collections import defaultdict
from agent import Agent

class HUD(Agent):

    # TODO left off here
    def accumulator_charge(self) -> int:
        resp = self.send("STAT POWR")
        parts = resp.split()
        return int(parts[2])


    def get_research_status(self) -> Dict[str, str]:
        research_status = {}

        while True:
            resp = self.send("LIST RESR")
            parts = resp.split()
            research = parts[0]
            status = parts[1]
            if research_status.get(research):
                return research_status
            else:
                research_status[research] = status
        

    def completed_research(self) -> List[str]:
        research_status = self.get_research_status()
        return [r for (r, s) in research_status.items() if s == "RESEARCHED"]


    def get_existing_agents(self) -> Dict[str, List[int]]:
        existing_agents = defaultdict(list)

        while True:
            resp = self.send("LIST AGNT")
            parts = resp.split()
            port = int(parts[0])
            kind = parts[1]

            if port in existing_agents[kind]:
                return existing_agents
            else:
                existing_agents[kind].append(port)
