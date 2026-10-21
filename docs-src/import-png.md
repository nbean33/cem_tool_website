# Importing artwork
> Trace a copper layout from a PNG image. The import dialog calibrates the image scale, masks one color as conductor, and places the result on a layer as a bitmap primitive that rasterizes into the mesh like any other shape.

## Opening the import dialog

Two entry points open the same dialog:

- `File` then `Import` then `Import PNG...` in the menu bar.
- Import PNG in the editor's Import dropdown beside the other artwork and project importers.

Both prompt for a `.png` file, then open the scale-and-mask dialog on that image. An image whose decoded RGBA buffer is at least 256 MB first shows a size warning with the estimated peak RAM and added project-file cost, and lets you cancel before loading.

## The steps

The image fills the canvas on the left. Wheel zooms, right-drag pans, and Fit View frames the whole image. The side panel walks the steps in order. OK stays disabled until both a mask color and a ruler length are set.

- **Crop.** Drag the yellow handles on the image to trim the region. Pixels outside the yellow rectangle are ignored. The default rectangle is the whole image.
- **Pick mask color.** Toggle Pick Mask Color, then click a pixel. That pixel's RGB becomes the mask color. Only pixels matching it are kept as conductor.
- **Adjust tolerance.** A per-channel RGB match window, 0 to 255. 0 matches the mask color exactly, higher values include more nearby colors. The overlay updates live: matched pixels fill purple, unmatched pixels drop out.
- **Enter real-world ruler length (mm).** Drag the two ruler endpoints across a feature whose true length you know, then type that length in mm. The dialog divides it by the measured pixel span to get mm-per-pixel. Snap to pixel edges locks the endpoints to pixel boundaries.
- **Choose target layer.** The layer the bitmap is placed on, defaulting to the active layer. A bitmap is a conductor pattern, so target the conductor layer the artwork belongs to.

The Ruler readout shows the current ruler span in pixels. The dialog warns if the mask selects zero pixels or nearly the whole image, both signs the color or tolerance is off.

!note **Scale accuracy.** Measure the longest feature you can. Each ruler endpoint is uncertain by about a pixel, so a short ruler carries far more scale error than a long one. Start from the highest-resolution image you have.

## What mask color to pick

The mask is a single color plus a tolerance, so the image should be close to monochrome artwork: one flat color for copper against a contrasting background. The picked color is the copper, whether it is the dark or the light region. A wider tolerance accepts colors farther from the picked one, absorbing the pixel-to-pixel variation of a noisy or JPEG-compressed image.

## Where the bitmap is placed

On OK, the masked pixels become a `bitmap` primitive on the chosen layer, recolored to that layer's editor color. The image is embedded in the project as pixel data, so it saves inside the `.rfsim` and needs no sidecar file. The bitmap is placed past the right edge of the existing geometry (or at the origin on an empty canvas), snapped to the grid, and selected. Its mm-per-pixel scale sets its physical size. Move, rotate, or flip it afterward like any other primitive.

At simulation export the bitmap rasterizes onto its layer's plane: every kept pixel becomes conductor at the layer's Z, the same conductor mask a drawn polygon would produce. A bitmap that has lost its pixels is a hard export error naming the primitive, never a silently blank layer.

## Reference-trace overlays

Importing a measured sweep to plot against a simulated result is a separate flow. Touchstone and CSV traces load on the results view, not here. See [S-parameters](docs-s-parameters.html).
