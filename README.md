## DrawBox
A sdf (signed distance feild or function) based 2d drawing/animation tool. 
Draw by inserting objects, moving them around and changing their parameters.
The cool thing with sdf's is that you can change objects "blob" value to blend them together.

## Installation

Install cargo:
// todo

Run:
``` git clone https://github.com/TageDan/drawBox ```

## Technical details
The project is written in rust using eframe/egui for ui and glow/GLSL for rendering.

## Future / Under construction
- Rotation
- Adjustable canvas dimensions
- Animation support.
- Grouping objects (independent sdf scope, grouped position/rotation).
- Save/Load
- Exporting as video/image
