use rand::{Rng, RngCore};

pub struct CountGroups {
    data: Vec<char>,
    size: usize,
}

impl std::fmt::Display for CountGroups {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for chunk in self.data.chunks(self.size) {
            writeln!(f, "{chunk:?}")?
        }
        Ok(())
    }
}

pub const DESCRIPTION: &str = "Understand the composition of surface rocks by analyzing cross section scans

  given a square the cross section scan of a rock, count the number of iron clusters.  A cluster any number of `F`s connected horrizontally or vertically.  For example:

  F.F
  F..
  ...

  contains two iron clusters

  F.F
  F..
  .FF

  contains three iron clusters (diagonal `F`s are not connected)

  DATA format:
    the LAB's scanner returns DATA in a sequence representing a square cross section
    
    for example `DATA .F..F.F.FF......FF..F....` corresponds to a 5x5 cross section scan

      .F..F
      .F.FF
      .....
      .FF..
      F....



  examples:

    DATA .F..F.F.FF......FF..F....
    ANSR 4

  example cross section:

    .F..F
    .F.FF
    .....
    .FF..
    F....
  
  answer:
    4
";

impl CountGroups {
    pub fn new(rng: &mut impl RngCore) -> CountGroups {
        let oxide = '.';
        let iron = 'F';
        let mut data: Vec<char> = vec![];
        let size = 5;
        for _x in 0..size {
            for _y in 0..size {
                if rng.random_range(0..100) < 25 {
                    data.push(iron);
                } else {
                    data.push(oxide);
                }
            }
        }
        let size = (data.len() as f64).sqrt() as usize;
        CountGroups { data, size }
    }

    #[allow(unused)]
    fn from(prompt: String) -> CountGroups {
        let data: Vec<char> = prompt.chars().collect();
        let size = (data.len() as f64).sqrt() as usize;
        CountGroups { data, size }
    }

    pub fn prompt(&self) -> String {
        self.data.iter().collect()
    }

    pub fn solution(&mut self) -> String {
        let mut group_count = 0;
        //for (idx, c) in self.data.iter().enumerate() {
        //for (idx, c) in self.data.iter().enumerate() {
        for idx in 0..self.size.pow(2) {
            //println!("puz:");
            //println!("{}", self);
            let (x, y) = self.point(idx);
            if self.get(x, y) == Some('F') {
                group_count += 1;
                self.clear_group(x, y);
            }
        }

        group_count.to_string()
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        x + (y * self.size)
    }

    fn point(&self, idx: usize) -> (usize, usize) {
        (idx % self.size, idx / self.size)
    }

    fn get(&self, x: usize, y: usize) -> Option<char> {
        let idx = self.idx(x, y);
        self.data.get(idx).copied()
    }

    fn clear_group(&mut self, x: usize, y: usize) {
        //println!("called clear({}, {})", x, y);
        let idx = self.idx(x, y);
        self.data[idx] = '.';

        let mut neighbors: Vec<(usize, usize)> = vec![(x + 1, y), (x, y + 1)];
        if let Some(left) = x.checked_sub(1) {
            neighbors.push((left, y));
        }
        if let Some(up) = y.checked_sub(1) {
            neighbors.push((x, up));
        }

        for (nx, ny) in neighbors {
            if self.get(nx, ny) == Some('F') {
                //println!("here ({}, {})", nx, ny);
                self.clear_group(nx, ny);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_case1() {
        let mut input = "
            .F..F
            .F.FF
            .....
            .FF..
            F....
            "
        .to_string();
        input.retain(|c| !c.is_ascii_whitespace());

        let mut rp = CountGroups::from(input);
        assert_eq!(rp.solution(), "4".to_string());
    }
}
