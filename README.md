# four
💎 A 4-dimensional renderer.

<p>
  <img src="https://github.com/mwalczyk/four/blob/master/screenshots/screenshot.gif" alt="screenshot" width="300" height="auto"/>
</p>

## Description
After seeing videos of [Miegakure](http://miegakure.com/) gameplay, I became very interested in the idea of 4-dimensional rendering. It turns out there are several ways to visualize 4D objects. Perhaps the simplest method involves a projection from 4D to 3D. This is similar to how "typical" 3D engines display objects onto the surface of a 2D screen (your display). Similar to this classic 3D -> 2D projection, the 4D -> 3D projection can either be a perspective projection or parallel projection (orthographic). 

Most people are familiar with this form of visualization. When a 4D -> 3D perspective projection is applied to the hypercube (or tesseract), the result is the familiar animation of an "outer" cube with a smaller, nested "inner" cube that appears to unfold itself via a series of 4-dimensional plane rotations. If you are interested in this method of visualization, I highly recommend Steven Hollasch's [thesis](http://hollasch.github.io/ray4/Four-Space_Visualization_of_4D_Objects.html#chapter4), titled "Four-Space Visualization of 4D Objects".

Another way to visualize 4-dimensional objects is via a "slicing" procedure, which produces a series of 3-dimensional cross-sections of the full polytope. This is analogous to cutting a 3D polyhedra with a plane (think MRI scans). Much of the math carries over to 4D, and this is how Miegakure renders objects. In order to facilitate this process, meshes in `four` are first decomposed into a set of tetrahedrons. This is similar to how we can decompose the faces of a regular polyhedron into triangles. Why do we do this? It turns out that slicing tetrahedrons in 4-dimensions is much simpler than slicing the full cells that make up the "face" of a polytope. In particular, a sliced tetrahedron (embedded in 4-dimensions) will always produce zero, 3, or 4 vertices, which makes things quite a bit easier.

NOTE: this project will only run on graphics cards that support OpenGL [Direct State Access](https://www.khronos.org/opengl/wiki/Direct_State_Access) (DSA).

## Tested On
- Windows 8.1, Windows 10
- NVIDIA GeForce GTX 970M, NVIDIA GeForce GTX 980
- Rust compiler version `1.21.0`

## To Build
1. Clone this repo.
2. Make sure 🦀 [Rust](https://www.rust-lang.org/en-US/) installed and `cargo` is in your `PATH`.
3. Inside the repo, run: `cargo build --release`.

## To Do
- [ ] Implement a more generic approach to deriving a polytope's H-representation based on its dual
- [ ] Implement a GPU-based slicing procedure (probably using compute shaders)
- [ ] Add additional polytopes (8-cell, 24-cell, 600-cell, etc.)
- [ ] Add 4-dimensional "extrusion" (i.e. things like spherinders)
- [ ] Add "hollow"-cell variants of each polytope
- [ ] Add shadow maps and diffuse lighting

## Credits
Some of the `.txt` shape files are from Paul Bourke's [website](http://paulbourke.net/geometry/hyperspace/). Eventually, these shapes will (hopefully) be generated procedurally, but for now, they are loaded offline. 

Thanks to [@GSBicalho](https://github.com/GSBicalho) and [@willnode](https://github.com/willnode) for their guidance throughout this project. Their responses to my questions were vital towards my understanding the 4D slicing procedure. 

I would like to give a special "thank you" to the author of [Eusebeia](http://eusebeia.dyndns.org/4d/), who answered many, many of my (probably stupid) questions, provided a more accurate `.txt` file for the 120-cell, and helped me understand much of the necessary math that was required in order to complete this project. If you are interested in higher-dimensional rendering, I would encourage you to check out their site!

### License

:copyright: The Interaction Department 2018

[Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/)
