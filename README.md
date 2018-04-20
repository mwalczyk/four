# Four
A 4D rendering engine.

<p>
  <img src="https://github.com/mwalczyk/four/blob/master/screenshots/screenshot.gif" alt="screenshot" width="300" height="auto"/>
</p>

## Description
This is largely a personal research project into the realm of 4-dimensional rendering.

This project will only run on graphics cards that support OpenGL [Direct State Access](https://www.khronos.org/opengl/wiki/Direct_State_Access) (DSA).

## Tested On
Windows 8.1, NVIDIA GeForce GTX 970M.

## To Build
1. Clone this repo.
2. Run `cargo build --release`.

## Credits
The `.txt` shape files are from Paul Bourke's [website](http://paulbourke.net/geometry/hyperspace/). Eventually, these shapes will (hopefully) be generated procedurally, but for now, they are loaded offline. 

Thanks to [@GSBicalho](https://github.com/GSBicalho) for his guidance throughout this project. His responses to my questions were vital towards my understanding the 4D slicing procedure. 

### License

:copyright: The Interaction Department 2018

[Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/)
