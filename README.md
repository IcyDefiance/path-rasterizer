# WIP Path Rasterizer for WGPU

[View the glorious screenshot.](https://raw.githubusercontent.com/IcyDefiance/path-rasterizer/main/assets/screenshot.png)

This library only supports straight lines and quadratic bezier curves so far, and it's not quite optimal yet. It also avoids compute shaders for now, so it should work in a browser.

Lyon is used with infinite tolerance for interior tessellation, and a few more triangles are added to render curves with Loop and Blinn's techniques, described [here](https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-25-rendering-vector-art-gpu).
