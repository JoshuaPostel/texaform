<p align="center"><b>Notice:</b> Texaform is in a pre-alpha state.  Feedback is greatly appreciated.</p>

<h1 align="center">TEXAFORM</h1>

<p align="center">A factory game automated by your code</p>

![](assets/demo.gif)

<p align="center">
  <img src="/assets/main_menu.png" width="400" />
  <img src="/assets/surface.png" width="400" />
  <img src="/assets/tech_tree.png" width="400" />
  <img src="/assets/documentation.png" width="400" />
</p>

Von Neumann probe 147 just landed on exoplanet TOI-1846 b.  Control agents on the planet's surface by sending simple text commands over [TCP](https://en.wikipedia.org/wiki/Transmission_Control_Protocol). Gather resources, process intermediates, conduct research, and produce more agents.

## To Run

```
git clone git@github.com:JoshuaPostel/texaform.git
cd texaform
cargo run --release
```

The Texaform user interface runs in the terminal.  Appearance will vary based on terminal font and colors.


Texaform will use local ports 3333 and upward (one per agent) for TCP communication with your code.  Ports 3333-3335 must not be in use when launching texaform.

## Influences

* [Factorio](https://www.factorio.com/)
* [Exapunks](https://www.zachtronics.com/exapunks/) 
* [Advent of Code](https://adventofcode.com/)
