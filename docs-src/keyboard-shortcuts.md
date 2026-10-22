# Keyboard shortcuts
> Every keyboard shortcut in Machshev Labs, grouped by area. Drawing, constraint, and canvas keys act on the Edit tab only, and every single-letter key is ignored while a text field or spin box has focus so typing a value never arms a tool.

## Application

These work from any tab.

Key || Action
`Ctrl+N` || New project
`Ctrl+O` || Open project
`Ctrl+S` || Save
`Ctrl+Shift+S` || Save As
`Ctrl+Z` || Undo
`Ctrl+Y` || Redo
`Ctrl+Shift+Z` || Redo

## Drawing tools

Each key selects the matching tool in the Edit tab toolbar. The Arc, Cutout, More Ports, Import PNG, and Script Shape tools have no key and are reached from the toolbar.

Key || Tool
`Esc` || Select (also cancels the active tool)
`R` || Rectangle
`C` || Circle
`P` || Polygon
`F` || Via
`O` || Hole
`Q` || Point port
`A` || Area port
`M` || Ruler

## Constraint and dimension tools

Each key arms a pick tool that then waits for you to click the anchors, edges, or rims it needs.

Key || Tool
`D` || Dimension
`I` || Coincident
`H` || Horizontal
`V` || Vertical
`T` || Tangent
`N` || Angle
`K` || Fix
`E` || Equal

## Editing the selection

These act on the current selection in the Edit tab canvas. Cut and paste are blocked in viewer mode, copy stays available.

Key || Action
`Ctrl+A` || Select all shapes
`Delete` || Delete the selection
`Backspace` || Delete the selection
`Space` || Rotate the selection 90 deg clockwise
`X` || Mirror the selection horizontally
`Y` || Mirror the selection vertically
`Ctrl+C` || Copy
`Ctrl+X` || Cut
`Ctrl+V` || Paste

While placing a port, Space or R rotates the port ghost 90 deg instead of rotating a selection.

## Canvas, grid, and layers

Key || Action
`S` || Toggle snap to grid
`Shift+S` || Cycle the view mode
`-` || Shrink the grid spacing one step
`=` || Grow the grid spacing one step
`+` || Grow the grid spacing one step
`1` || Select layer 1 (through 9 for layer 9)

## Stackup layers

These act on the Stackup layer list when it has focus, and on the layer bars in Mesh View.

Key || Action
`Delete` || Delete the selected layer
`Ctrl+Up` || Move the selected layer up one place
`Ctrl+Down` || Move the selected layer down one place

In Mesh View, Up and Down select the layer above or below.

## RF calculators

Key || Action
`Ctrl+R` || Toggle the RF calculators overlay

## S-parameter markers

These work on the S-parameters plot.

Key || Action
`M` || Toggle marker placement
`Esc` || Cancel marker placement
`Delete` || Delete the selected marker
`Backspace` || Delete the selected marker

## 3D views

The 3D mesh preview, the Radiation Pattern tab, and the Field Viewer share one set of mouse controls.

Input || Action
Left drag || Orbit
Right drag, middle drag, or Shift plus left drag || Pan
Wheel || Zoom toward the point under the pointer
`Esc` || End the drag
