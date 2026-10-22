# Variables, constraints, and generators
> The parametric system drives geometry by relationships instead of fixed numbers: named variables, a constraint solver on the canvas, and generators that stamp copies of a master shape. One change propagates everywhere it is referenced.

## Variables

Variable Studio holds the project's named variables. Each row is a name, a value, and an optional min and max.

!fig Variable Studio panel

A value is a literal number or an expression over other variables. The Value cell tooltip shows the resolved number, and the Name cell tooltip shows how many fields use the variable and its group. A name is letters, digits, and underscore, does not start with a digit, and is case sensitive. `pi`, `e`, and the function names are reserved. Min and Max are an optional allowed range. Once both are filled, the only check is that min does not exceed max. Set a variable's group from the row context menu.

Any numeric field of a shape can bind to an expression. To bind a field, type a variable name or an expression straight into its box, or right-click the box and pick a variable from the Variable menu. A polygon vertex binds the same way in the Vertices table, and its expression is measured from the shape's origin. A bound field reads its value from the expression each time the project resolves rather than storing a fixed number. Categorical fields (layer names, port subtype, feed direction) never bind. A binding addresses one field: a shape param, a layer property, a domain or FDTD setting, or a generator param.

The expression grammar: numbers, variable names, `+ - * / // % **` with unary sign and parentheses, the functions `sin cos tan asin acos atan atan2 sqrt abs min max floor ceil round exp log log10`, and the constants `pi` and `e`. Trig is in degrees, the CAD convention, so sin, cos, and tan take degrees and the inverse functions return degrees. An expression evaluates in the field's own unit with no unit suffix. A dependency cycle, an undefined name, or a result of infinity or NaN is reported with the offending token named, never applied.

Set values three ways: edit the cell in Variable Studio, or headless with `machshev project set --var name=value --project p.rfsim`, or per run with `machshev run --set name=value`. Both CLI paths re-resolve the whole parametric system (variables, bindings, then constraints) before the project is used. `--set` applies at run start.

## Constraints

The constraint solver holds geometric relationships as you draw and drag. Add a relation and the solver drives it to satisfaction in a least-squares sense, keeping every enabled constraint true. Dimensions and relation glyphs draw on the canvas next to the shapes they bind.

Each constraint type:

Constraint || Meaning
`coincident` || Anchor onto an anchor, an edge (point-on-line), or a circle rim.
`point_on_line` || An anchor lies on an edge segment.
`point_on_circle` || An anchor lies on a circle or arc rim.
`horizontal` || Two anchors share a Y coordinate, or an anchor shares Y with a Y-axis mesh pin.
`vertical` || Two anchors share an X coordinate, or an anchor shares X with an X-axis mesh pin.
`distance` || Distance between two anchors or an edge length (aligned, horizontal, or vertical). Horizontal mode also spans an anchor or another pin to an X-axis mesh pin, vertical mode to a Y-axis pin. The value is a number or expression.
`angle` || Angle between two edges, in degrees.
`radius` || Radius of a circle, arc, or via ring.
`diameter` || Diameter of a circle, arc, or via ring.
`equal` || Two refs measure the same: edge lengths, circle/arc radii, or anchor-pair distances (four-point form).
`tangent` || A rect/poly edge tangent to a circle/arc, or circle tangent to circle.
`fix` || Pin an anchor, a whole shape, or a mesh pin in place.

A dimensional constraint (distance, angle, radius, diameter) carries a target value that is a number or an expression over variables, so a dimension can be driven by a variable. A redundant but consistent dimension is downgraded to a driven reference dimension: it reports its measured value and is excluded from the solve. An equal constraint ties two edge lengths or two circle radii together.

Arm a tool from the keyboard, then click the anchors, edges, or rims it relates. Each key toggles its tool, and it stays armed until Escape, another tool, or Select disarms it.

Key || Tool
`D` || Dimension
`I` || Coincident
`H` || Horizontal
`V` || Vertical
`T` || Tangent
`N` || Angle
`K` || Fix
`E` || Equal

Dragging is constrained. When you move a shape that carries constraints, the solver runs on release with the dragged shape pinned toward where you dropped it and its constrained partners moved the least amount that keeps the hard constraints satisfied. An infeasible drop lands at the closest feasible pose instead of reverting the whole move. A shape held by fix does not move.

## Generators

A generator stamps copies of one or more master shapes. Select the masters, then create an array from the Generators panel.

!fig Generators panel

The stored state is the master shapes plus the generator parameters. Copies (instances) are never written into the saved shape list. They materialize at render time so the canvas shows them, and again at export time for the solver, always from the current masters. Editing a master updates its copies live: a width change on the source resizes every copy at once, and dragging the source moves the copies with it, holding their relative offsets.

The two generator types:

Generator || Params || Meaning
`array` || `nx, ny, nx_minus, ny_minus, dx_mm, dy_mm, mode, lock_spacing` || Rectangular grid of copies of one or more master shapes. The master sits at index (0, 0) and the grid grows both ways along each axis.
`linked_clone` || `dx_mm, dy_mm` || A single offset copy of a master shape (the degenerate array case).

There is no instance limit. A grid is bounded by the machine's memory, and a build that runs out says so.

`nx`, `ny`, `dx_mm`, and `dy_mm` each accept a number or an expression, so a variable can drive the copy count or the pitch.

!fig EBG array on the canvas

A generator has no limit on its copy count. A large array draws on the canvas behind a loading overlay. Cancel stops the drawing and removes the copies drawn so far, and the Console says so. The run and the exports still build every copy.
