"""Helpers shared by the Ronnie 3000 skin scripts: live knobs, alpha cleanup, knob filmstrips
and the skin manifest.

A skin script builds its gear, calls `knob_rig.add(...)` for each knob, and ends with
`finish(scene, out_dir, skin_id, samples)`, which writes:

- `body.png`: the gear on a transparent background, with no haze at the frame edges;
- `knob-<control>-NN.png`: FRAMES crops around each live knob, from value 0 to value 1,
  feathered to a circle so that they sit on the body without a seam;
- `skin.json`: the size of the body and where each knob sits on it, in body pixels.
"""
import json
import math
import os

import bpy
import numpy as np
from bpy_extras.object_utils import world_to_camera_view
from mathutils import Vector

FRAMES = 32
# The body fits this height, and no more than MAX_WIDTH, so that a 300-point stage stays sharp at 2x.
TARGET_HEIGHT = 640
MAX_WIDTH = 1100
# A knob crop is this many knob radii wide; the feather runs over its outer FEATHER part.
CROP_RADII = 2.7
FEATHER = 0.12
# Shadow-catcher haze below this alpha is dropped; alpha fades to 0 within EDGE_FADE px of the frame.
ALPHA_FLOOR = 4.0 / 255.0
EDGE_FADE = 12

LABEL_CONTROL = {"GAIN IN": "gain", "OUTPUT": "output", "MIX": "mix"}


class Knob:
    def __init__(self, label, center, radius, axis, movers):
        self.label = label
        self.control = LABEL_CONTROL[label]
        self.center = center
        self.radius = radius
        self.axis = axis
        # (object, distance of its centre from the knob axis, local z, length offset) per moving part.
        self.movers = movers
        self.local_center = None

    def set_value(self, value):
        ang = math.radians(225 - 270 * value)
        cx, cy = self.local_center
        for obj, dist, _ in self.movers:
            obj.location.x = cx + math.cos(ang) * dist
            obj.location.y = cy + math.sin(ang) * dist
            obj.rotation_euler = (0, 0, ang - math.pi / 2)


class KnobRig:
    def __init__(self):
        self.knobs = []

    def add(self, label, local_center, top_world, radius, axis, movers):
        """Registers a live knob. `local_center` is (x, y) in the space the movers live in."""
        if label not in LABEL_CONTROL:
            return
        knob = Knob(label, top_world, radius, axis, movers)
        knob.local_center = local_center
        self.knobs.append(knob)


knob_rig = KnobRig()


def fit_resolution(scene):
    """Scales the render so that the body is TARGET_HEIGHT tall and at most MAX_WIDTH wide."""
    r = scene.render
    pct = TARGET_HEIGHT * 100.0 / r.resolution_y
    pct = min(pct, MAX_WIDTH * 100.0 / r.resolution_x)
    r.resolution_percentage = max(1, int(round(pct)))
    return effective_size(scene)


def effective_size(scene):
    r = scene.render
    return (int(r.resolution_x * r.resolution_percentage / 100),
            int(r.resolution_y * r.resolution_percentage / 100))


def project(scene, cam, point):
    """Pixel position of a world point, x from the left and y from the top."""
    w, h = effective_size(scene)
    co = world_to_camera_view(scene, cam, Vector(point))
    return co.x * w, (1.0 - co.y) * h


def load_pixels(path):
    img = bpy.data.images.load(path, check_existing=False)
    w, h = img.size
    px = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    # Blender stores rows from the bottom up.
    return img, px.reshape(h, w, 4)


def save_pixels(img, px, path):
    img.pixels.foreach_set(px.ravel())
    img.filepath_raw = path
    img.file_format = "PNG"
    img.save()
    bpy.data.images.remove(img)


def clean_body(path):
    """Drops the shadow-catcher haze and fades alpha to 0 at the frame edges."""
    img, px = load_pixels(path)
    h, w, _ = px.shape
    a = px[..., 3]
    clean = np.clip((a - ALPHA_FLOOR) / (1.0 - ALPHA_FLOOR), 0.0, 1.0)
    ys = np.arange(h, dtype=np.float32)[:, None]
    xs = np.arange(w, dtype=np.float32)[None, :]
    edge = np.minimum(np.minimum(xs, w - 1 - xs), np.minimum(ys, h - 1 - ys))
    # The outer 2 px are fully clear, then alpha ramps up over EDGE_FADE px.
    fade = np.clip((edge - 1.0) / EDGE_FADE, 0.0, 1.0)
    # PNG pixels load with straight alpha, so only the alpha channel changes.
    px[..., 3] = clean * fade
    save_pixels(img, px, path)


def feather(path):
    """Fades a knob crop to a circle so that it blends into the body."""
    img, px = load_pixels(path)
    h, w, _ = px.shape
    ys = np.arange(h, dtype=np.float32)[:, None] - (h - 1) / 2.0
    xs = np.arange(w, dtype=np.float32)[None, :] - (w - 1) / 2.0
    d = np.sqrt(xs * xs + ys * ys)
    outer = min(w, h) / 2.0
    inner = outer * (1.0 - FEATHER)
    mask = np.clip((outer - d) / max(outer - inner, 1.0), 0.0, 1.0)
    px[..., 3] *= mask
    save_pixels(img, px, path)


def render_to(scene, path):
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)


def finish(scene, out_dir, skin_id, samples):
    os.makedirs(out_dir, exist_ok=True)
    cam = scene.camera
    width, height = fit_resolution(scene)
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.use_persistent_data = True

    scene.cycles.samples = samples
    body = os.path.join(out_dir, "body.png")
    render_to(scene, body)
    clean_body(body)

    bpy.context.view_layer.update()
    right = cam.matrix_world.to_3x3().col[0].normalized()
    knobs = []
    for knob in knob_rig.knobs:
        center = Vector(knob.center() if callable(knob.center) else knob.center)
        cx, cy = project(scene, cam, center)
        ex, ey = project(scene, cam, center + right * knob.radius)
        radius_px = math.hypot(ex - cx, ey - cy)
        side = int(math.ceil(radius_px * CROP_RADII / 2.0)) * 2
        x0 = int(round(cx - side / 2))
        y0 = int(round(cy - side / 2))
        x0 = max(0, min(width - side, x0))
        y0 = max(0, min(height - side, y0))
        r = scene.render
        r.use_border = True
        r.use_crop_to_border = True
        # Blender truncates each border edge to a pixel, so each edge sits a quarter pixel past
        # its target and float error cannot move it a pixel down.
        r.border_min_x = (x0 + 0.25) / width
        r.border_max_x = (x0 + side + 0.25) / width
        r.border_min_y = (height - y0 - side + 0.25) / height
        r.border_max_y = (height - y0 + 0.25) / height
        scene.cycles.samples = max(32, samples // 2)
        for i in range(FRAMES):
            knob.set_value(i / (FRAMES - 1))
            frame = os.path.join(out_dir, f"knob-{knob.control}-{i:02d}.png")
            render_to(scene, frame)
            size = tuple(bpy.data.images.load(frame, check_existing=False).size)
            assert size == (side, side), f"{knob.control} frame is {size}, not {side}"
            feather(frame)
        r.use_border = False
        r.use_crop_to_border = False
        knobs.append({
            "control": knob.control,
            "label": knob.label,
            "center": [x0 + side / 2.0, y0 + side / 2.0],
            "side": side,
            "frames": FRAMES,
            "columns": 8,
        })

    manifest = {"id": skin_id, "width": width, "height": height, "knobs": knobs}
    with open(os.path.join(out_dir, "skin.json"), "w") as f:
        json.dump(manifest, f, indent=2)
    print("WROTE", out_dir)
