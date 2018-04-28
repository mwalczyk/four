# four
💎 A 4-dimensional renderer.

<p>
  <img src="https://github.com/mwalczyk/four/blob/master/screenshots/screenshot.gif" alt="screenshot" width="300" height="auto"/>
</p>

## Description
After seeing videos of [Miegakure](http://miegakure.com/) gameplay, I became very interested in the idea of 4-dimensional rendering. It turns out there are several ways to visualize 4D objects. Perhaps the simplest method involves a projection from 4D to 3D. This is similar to how "typical" 3D engines display objects onto the surface of a 2D screen (your display). Similar to this classic 3D -> 2D projection, a 4D -> 3D projection can either be a perspective projection or parallel projection (orthographic). 

Most people are familiar with this form of visualization. When a 4D -> 3D perspective projection is applied to the hypercube (or [tesseract](https://en.wikipedia.org/wiki/Tesseract)), the result is the familiar animation of an "outer" cube with a smaller, nested "inner" cube that appears to unfold itself via a series of 4-dimensional plane rotations. The reason why this inner cube appears small is because it is "further away" in the 4-th dimensional (the `w`-axis of our 4D coordinate system). If you are interested in this form of visualization, I highly recommend Steven Hollasch's [thesis](http://hollasch.github.io/ray4/Four-Space_Visualization_of_4D_Objects.html#chapter4), titled _Four-Space Visualization of 4D Objects_.

Another way to visualize 4-dimensional objects is via a "slicing" procedure, which produces a series of 3-dimensional cross-sections of the full polytope. This is analogous to cutting a 3D polyhedra with a plane (think MRI scans). Much of the math carries over to 4D, and this is how Miegakure renders objects. In order to facilitate this process, meshes in `four` are first decomposed into a set of tetrahedrons. This is similar to how we can decompose the faces of a regular polyhedron into triangles. In particular, [any 3D convex polyhedron can be decomposed into tetrahedrons](https://mathoverflow.net/questions/7647/break-polyhedron-into-tetrahedron) by first subdividing its faces into triangles. Next, we pick a vertex from the polyhedron (any vertex will do). We connect all of the other face triangles to the chosen vertex to form a set of tetrahedrons (obviously ignoring faces that contain the chosen vertex). This is not necessarily the "minimal tetrahedral decomposition" of the polyhedron (which is an active area of research for many polytopes), but it always works.

So, we start with our 4D polytope, whose "faces" (usually referred to as "cells") are themselves convex polyhedra embedded in 4-dimensions. One by one, we decompose each of the cells into tetrahedra, and together, the sum of these tetrahedra form our 4-dimensional mesh. For example, the hypercube has 8 cells, each of which is a cube (this is why the hypercube is often called the 8-cell). Each cube produces 6 distinct tetrahedra, so all together, a hypercube will be decomposed into 48 tetrahedra.

Why do we do this? It turns out that slicing tetrahedrons in 4-dimensions is much simpler than slicing the full cells that make up the "face" of a polytope. In particular, a sliced tetrahedron (embedded in 4-dimensions) will always produce zero, 3, or 4 vertices, which makes things quite a bit easier.

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
- [ ] Research Munsell color solids

## Credits
Some of the `.txt` shape files are from Paul Bourke's [website](http://paulbourke.net/geometry/hyperspace/). Eventually, these shapes will (hopefully) be generated procedurally, but for now, they are loaded offline. 

Thanks to [@GSBicalho](https://github.com/GSBicalho) and [@willnode](https://github.com/willnode) for their guidance throughout this project. Their responses to my questions were vital towards my understanding the 4D slicing procedure. 

I would like to give a special "thank you" to the author of [Eusebeia](http://eusebeia.dyndns.org/4d/), who answered many, many of my (probably stupid) questions, provided a more accurate `.txt` file for the 120-cell, and helped me understand much of the necessary math that was required in order to complete this project. If you are interested in higher-dimensional rendering, I would encourage you to check out their site!

### License

:copyright: The Interaction Department 2018

[Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/)
