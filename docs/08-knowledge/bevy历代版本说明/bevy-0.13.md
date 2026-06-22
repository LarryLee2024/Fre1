# Bevy 0.13

## Posted on February 17, 2024 by Bevy Contributors

![Gameplay from Jarl, an in-production fantasy colony builder made with Bevy](https://bevy.org/news/bevy-0-13/jarl.webp)

[Gameplay from Jarl, an in-production fantasy colony builder made with Bevy](https://www.jarl-game.com/)

Thanks to **198** contributors, **672** pull requests, community reviewers, and our [**generous sponsors**](https://bevy.org/community/donate), we're happy to announce the **Bevy 0.13** release on [crates.io](https://crates.io/crates/bevy)!

For those who don't know, Bevy is a refreshingly simple data-driven game engine built in Rust. You can check out our [Quick Start Guide](https://bevy.org/learn/quick-start) to try it today. It's free and open source forever! You can grab the full [source code](https://github.com/bevyengine/bevy) on GitHub. Check out [Bevy Assets](https://bevy.org/assets) for a collection of community-developed plugins, games, and learning resources. And to see what the engine has to offer hands-on, check out the entries in the [latest Bevy Jam](https://itch.io/jam/bevy-jam-4/entries), including the winner [That's a LOT of beeeeees](https://andrewb330.itch.io/thats-a-lot-of-beeeeees).

To update an existing Bevy App or Plugin to **Bevy 0.13**, check out our [0.12 to 0.13 Migration Guide](https://bevy.org/learn/migration-guides/0-12-to-0-13/).

Since our last release a few months ago we've added a _ton_ of new features, bug fixes, and quality of life tweaks, but here are some of the highlights:

- **Lightmaps:** A fast, popular baked global illumination technique for static geometry (baked externally in programs like The Lightmapper).
- **Irradiance Volumes / Voxel Global Illumination:** A baked form of global illumination that samples light at the centers of voxels within a cuboid (baked externally in programs like Blender).
- **Approximate Indirect Specular Occlusion**: Improved lighting realism by reducing specular light leaking via specular occlusion.
- **Reflection Probes**: A baked form of axis aligned environment map that allows for realistic reflections for static geometry (baked externally in programs like Blender)
- **Primitive shapes:** Basic shapes are a core building block of both game engines and video games: we've added a polished, ready-to-use collection of them!
- **System stepping:** Completely pause and advance through your game frame-by-frame or system-by-system to interactively debug game logic, all while rendering continues to update.
- **Dynamic queries:** Refining queries from within systems is extremely expressive, and is the last big puzzle piece for runtime-defined types and third-party modding and scripting integration.
- **Automatically inferred command flush points:** Tired of reasoning about where to put `apply_deferred` and confused about why your commands weren't being applied? Us too! Now, Bevy's scheduler uses ordinary `.before` and `.after` constraints and inspects the system parameters to automatically infer (and deduplicate) synchronization points.
- **Slicing, tiling and nine-patch 2D images:** Ninepatch layout is a popular tool for smoothly scaling stylized tilesets and UIs. Now in Bevy!
- **Camera-Driven UI**: UI entity trees can now be selectively added to _any_ camera, rather than being globally applied to all cameras, enabling things like split screen UIs!
- **Camera Exposure**: Realistic / "real world" control over camera exposure via EV100, f-stops, shutter speed, and ISO sensitivity. Lights have also been adjusted to make their units more realistic.
- **Animation interpolation modes:** Bevy now supports non-linear interpolation modes in exported glTF animations.

## Initial Baked Lighting [#](https://bevy.org/news/bevy-0-13/#initial-baked-lighting)

Computing lighting in real time is expensive; but for elements of a scene that never move (like rooms or terrain), we can get prettier lighting and shadows for cheaper by computing it ahead of time using **global illumination**, then storing the results in a "baked" form that never changes. Global illumination is a more realistic (and expensive) approach to lighting that often uses ray tracing. Unlike Bevy's default rendering, it takes light bouncing off of other objects into account, producing more realistic effects through the inclusion of indirect light.

### Lightmaps [#](https://bevy.org/news/bevy-0-13/#lightmaps)

authors: @pcwalton

![lightmaps](https://bevy.org/news/bevy-0-13/lightmap.jpg)

**Lightmaps** are textures that store pre-computed global illumination results. They have been a mainstay of real-time graphics for decades. **Bevy 0.13** adds initial support for rendering lightmaps computed in other programs, such as [The Lightmapper](https://github.com/Naxela/The_Lightmapper). Ultimately we would like to add support for baking lightmaps directly in Bevy, but this first step unlocks lightmap workflows!

Like the [lightmaps example](https://github.com/bevyengine/bevy/blob/main/examples/3d/lightmaps.rs) shows, just load in your baked lightmap image, and then insert a [`Lightmap`](https://docs.rs/bevy/0.13.0/bevy/pbr/struct.Lightmap.html) component on the corresponding mesh.

### Irradiance Volumes / Voxel Global Illumination [#](https://bevy.org/news/bevy-0-13/#irradiance-volumes-voxel-global-illumination)

authors: @pcwalton

![irradiance volume](https://bevy.org/news/bevy-0-13/irradiance_volume.jpg)

**Irradiance volumes** (or voxel global illumination) is a technique used for approximating indirect light by first dividing a scene into cubes (voxels), then sampling the amount of light present at the center of each of those voxels. This light is then added to objects within that space as they move through it, changing the ambient light level on those objects appropriately.

We've chosen to use the ambient cubes algorithm for this, based on Half Life 2. This allows us to match Blender's [Eevee renderer](https://docs.blender.org/manual/en/latest/render/eevee/index.html), giving users a simple and free path to creating nice-looking irradiance volumes for their own scenes.

Notice how this sphere subtly picks up the colors of the environment as it moves around, thanks to irradiance volumes:

For now, you need to use external tools such as Blender to bake irradiance volumes, but in the future we would like to support baking irradiance volumes directly in Bevy!

## Minimal Reflection Probes [#](https://bevy.org/news/bevy-0-13/#minimal-reflection-probes)

authors: @pcwalton

**Environment maps** are 2D textures used to simulate lighting, reflection, and skyboxes in a 3D scene. **Reflection probes** generalize environment maps to allow for multiple environment maps in the same scene, each of which has its own axis-aligned bounding box. This is a standard feature of physically-based renderers and was inspired by the [corresponding feature in Blender's Eevee renderer](https://docs.blender.org/manual/en/latest/render/eevee/light_probes/reflection_cubemaps.html).

In the [reflection probes PR](https://github.com/bevyengine/bevy/pull/11366), we've added basic support for these, laying the groundwork for pretty, high-performance reflections in Bevy games. Like with the baked global illumination work discussed above, these must currently be precomputed externally, then imported into Bevy. As discussed in the PR, there are quite a few caveats: WebGL2 support is effectively non-existent, sharp and sudden transitions will be observed because there's no blending, and all cubemaps in the world of a given type (diffuse or specular) must have the same size, format, and mipmap count.

![reflection probes](https://bevy.org/news/bevy-0-13/reflection_probes.jpg)

## Approximate Indirect Specular Occlusion [#](https://bevy.org/news/bevy-0-13/#approximate-indirect-specular-occlusion)

authors: @aevyrie

Bevy's current PBR renderer over-brightens the image, especially at grazing angles where the fresnel effect tends to make surfaces behave like mirrors. This over-brightening happens because the surfaces must reflect _something_, but without path traced or screen-space reflections, the renderer has to guess _what_ is being reflected. The best guess it can make is to sample the environment cube map, even if light would've hit something else before reaching the environment light. This artifact, where light occlusion is ignored, is called specular light leaking.

Consider a car tire; though the rubber might be shiny, you wouldn't expect it to have bright specular highlights inside a wheel well, because the car itself is blocking (occluding) the light that would otherwise cause these reflections. Fully checking for occlusion can be computationally expensive.

**Bevy 0.13** adds support for **Approximate Indirect Specular Occlusion**, which uses our existing [Screen Space Ambient Occlusion](https://bevy.org/news/bevy-0-11/#screen-space-ambient-occlusion) to _approximate_ specular occlusion, which can run efficiently in real time while still producing reasonably high quality results:

Drag this image to compare

![Specular Occlusion On](https://bevy.org/news/bevy-0-13/specular_occlusion_on.jpg)![Specular Occlusion Off](https://bevy.org/news/bevy-0-13/specular_occlusion_off.jpg)

Model Credits: [BMW R1200GS Motorcycle](https://sketchfab.com/3d-models/bmw-r1200gs-motorcycle-6550451b0ae547039585a44286b2f530) by Moto3D is licensed under [CC-BY-4.0](http://creativecommons.org/licenses/by/4.0/).

In the future, this could be further improved with screen space reflections (SSR). However, conventional wisdom is that you should use specular occlusion alongside SSR, because SSR still suffers from light leaking artifacts.

## Primitive Shapes [#](https://bevy.org/news/bevy-0-13/#primitive-shapes)

authors: @Jondolf, @NiseVoid, @aevyrie

Geometric shapes are used all across game development, from primitive mesh shapes and debug gizmos to physics colliders and raycasting. Despite being so commonly used across several domains, Bevy hasn't really had any general-purpose shape representations.

This is changing in **Bevy 0.13** with the introduction of first-party **primitive shapes**! They are lightweight geometric primitives designed for maximal interoperability and reusability, allowing Bevy and third-party plugins to use the same set of basic shapes and increase cohesion within the ecosystem. See the original [RFC](https://github.com/bevyengine/rfcs/blob/main/rfcs/12-primitive-shapes.md) for more details.

The built-in [collection of primitives](https://docs.rs/bevy/0.13.0/bevy/math/primitives/index.html) is already quite sizeable:

|2D|3D|
|---|---|
|[`Rectangle`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Rectangle.html)|[`Cuboid`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Cuboid.html)|
|[`Circle`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Circle.html)|[`Sphere`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Sphere.html)|
|[`Ellipse`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Ellipse.html)||
|[`Triangle2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Triangle2d.html)||
|[`Plane2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Plane2d.html)|[`Plane3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Plane3d.html)|
|[`Line2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Line2d.html)|[`Line3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Line3d.html)|
|[`Segment2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Segment2d.html)|[`Segment3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Segment3d.html)|
|[`Polyline2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Polyline2d.html), [`BoxedPolyline2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.BoxedPolyline2d.html)|[`Polyline3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Polyline3d.html), [`BoxedPolyline3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.BoxedPolyline3d.html)|
|[`Polygon`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Polygon.html), [`BoxedPolygon`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.BoxedPolygon.html)||
|[`RegularPolygon`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.RegularPolygon.html)||
|[`Capsule2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Capsule2d.html)|[`Capsule3d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Capsule3d.html)|
||[`Cylinder`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Cylinder.html)|
||[`Cone`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Cone.html)|
||[`ConicalFrustum`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.ConicalFrustum.html)|
||[`Torus`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Torus.html)|

[More primitives](https://github.com/bevyengine/bevy/issues/10572) will be added in future releases.

Some use cases for primitive shapes include meshing, gizmos, bounding volumes, colliders, and ray casting functionality. Several of these have landed in 0.13 already!

### Rendering [#](https://bevy.org/news/bevy-0-13/#rendering)

Primitive shapes can be rendered using both meshes and gizmos. In this section, we'll take a closer look at the new APIs.

Below, you can see a cuboid and a torus rendered using meshes and gizmos. You can check out all primitives that can be rendered in the new [Rendering Primitives](https://bevy.org/examples/Math/render-primitives) example.

![On the left: A cuboid rendered with gizmos. It consists of 12 white lines. On the right: A cuboid rendered with meshes. It consists of 6 white faces.](https://bevy.org/news/bevy-0-13/cuboids.png)

![On the left: A torus rendered with gizmos. It consists of many small rings, all connected by 4 big rings. On the right: A torus rendered with meshes. A shape that looks like a donut.](https://bevy.org/news/bevy-0-13/tori.png)

#### Meshing [#](https://bevy.org/news/bevy-0-13/#meshing)

authors: @Jondolf

Previous versions of Bevy have had types like [`Quad`](https://docs.rs/bevy/0.13.0/bevy/prelude/shape/struct.Quad.html), [`Box`](https://docs.rs/bevy/0.13.0/bevy/prelude/shape/struct.Box.html), and [`UVSphere`](https://docs.rs/bevy/0.13.0/bevy/prelude/shape/struct.UVSphere.html) for creating meshes from basic shapes. These have been deprecated in favor of a builder-like API using the new geometric primitives.

Primitives that support meshing implement the [`Meshable`](https://docs.rs/bevy/0.13.0/bevy/prelude/trait.Meshable.html) trait. For some shapes, the [`mesh`](https://docs.rs/bevy/0.13.0/bevy/prelude/trait.Meshable.html#tymethod.mesh) method returns a [`Mesh`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Mesh.html) directly:

```rust
let before = Mesh::from(Quad::new(Vec2::new(2.0, 1.0)));
let after = Rectangle::new(2.0, 1.0).mesh(); // Mesh::from also works
```

For most primitives however, it returns a builder for optional configuration:

```rust
// Create a circle mesh with a specified vertex count
let before = Mesh::from(Circle {
    radius: 1.0,
    vertices: 64,
});
let after = Circle::new(1.0).mesh().resolution(64).build();
```

Below are a few more examples of meshing with the new primitives.

```rust
// Icosphere
let before = meshes.add(
    Mesh::try_from(Icosphere {
        radius: 2.0,
        subdivisions: 8,
    })
    .unwrap()
);
let after = meshes.add(Sphere::new(2.0).mesh().ico(8).unwrap());

// Cuboid
// (notice how Assets::add now also handles mesh conversion automatically)
let before = meshes.add(Mesh::from(shape::Box::new(2.0, 1.0, 1.0)));
let after = meshes.add(Cuboid::new(2.0, 1.0, 1.0));

// Plane
let before = meshes.add(Mesh::from(Plane::from_size(5.0)));
let after = meshes.add(Plane3d::default().mesh().size(5.0, 5.0));
```

With the addition of the primitives, meshing is also supported for more shapes, like [`Ellipse`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Ellipse.html), [`Triangle2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Triangle2d.html), and [`Capsule2d`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Capsule2d.html). However, note that meshing is not yet implemented for all primitives, such as [`Polygon`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Polygon.html) and [`Cone`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Cone.html).

Below you can see some meshes in the [`2d_shapes`](https://bevy.org/examples/2D%20Rendering/2d-shapes/) and [`3d_shapes`](https://bevy.org/examples/3D%20Rendering/3d-shapes/) examples.

![An example with 2D mesh shapes](https://bevy.org/news/bevy-0-13/2d_shapes.png)

![An example with 3D mesh shapes](https://bevy.org/news/bevy-0-13/3d_shapes.png)

Some default values for mesh shape dimensions have also been changed to be more consistent.

#### Gizmos [#](https://bevy.org/news/bevy-0-13/#gizmos)

authors: @RobWalt

Primitives can also be rendered with [`Gizmos`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/struct.Gizmos.html). There are two new generic methods:

- [`gizmos.primitive_2d(primitive, position, angle, color)`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/trait.GizmoPrimitive2d.html)
- [`gizmos.primitive_3d(primitive, position, rotation, color)`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/trait.GizmoPrimitive2d.html)

Some primitives can have additional configuration options similar to existing [`Gizmos`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/struct.Gizmos.html) drawing methods. For example, calling [`primitive_3d`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/trait.GizmoPrimitive2d.html) with a [`Sphere`](https://docs.rs/bevy/0.13.0/bevy/prelude/struct.Sphere.html) returns a [`SphereBuilder`](https://docs.rs/bevy/0.13.0/bevy/gizmos/primitives/dim3/struct.SphereBuilder.html), which offers a `segments` method to control the level of detail of the sphere.

```rust
let sphere = Sphere { radius };
gizmos
    .primitive_3d(sphere, center, rotation, color)
    .segments(segments);
```

### Bounding Volumes [#](https://bevy.org/news/bevy-0-13/#bounding-volumes)

authors: @NiseVoid, @Jondolf

In game development, spatial checks have several valuable use cases, such as getting all entities that are in the camera's view frustum or near the player, or finding pairs of physics objects that might be intersecting. To speed up such checks, bounding volumes are used to approximate more complex shapes.

**Bevy 0.13** adds some new publicly available bounding volumes: [`Aabb2d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.Aabb2d.html), [`Aabb3d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.Aabb3d.html), [`BoundingCircle`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.BoundingCircle.html), and [`BoundingSphere`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.BoundingSphere.html). These can be created manually, or generated from primitive shapes.

Each bounding volume implements the [`BoundingVolume`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/trait.BoundingVolume.html) trait, providing some general functionality and helpers. The [`IntersectsVolume`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/trait.IntersectsVolume.html) trait can be used to test for intersections with these volumes. This trait is implemented for bounding volumes themselves, so you can test for intersections between them. This is supported between all existing bounding volume types, but only those in the same dimension.

Here is an example of how bounding volumes are constructed, and how an intersection test is performed:

```rust
// We create an axis-aligned bounding box that is centered at position
let position = Vec2::new(100., 50.);
let half_size = Vec2::splat(20.);
let aabb = Aabb2d::new(position, half_size);

// We create a bounding circle that is centered at position
let position = Vec2::new(80., 70.);
let radius = 30.;
let bounding_circle = BoundingCircle::new(position, radius);

// We check if the volumes are intersecting
let intersects = bounding_circle.intersects(&aabb);
```

There are also two traits for the generation of bounding volumes: [`Bounded2d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/trait.Bounded2d.html) and [`Bounded3d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/trait.Bounded3d.html). These are implemented for the new primitive shapes, so you can easily compute bounding volumes for them:

```rust
// We create a primitive, a hexagon in this case
let hexagon = RegularPolygon::new(50., 6);

let translation = Vec2::new(50., 200.);
let rotation = PI / 2.; // Rotation in radians

// Now we can get an Aabb2d or BoundingCircle from this primitive.
// These methods are part of the Bounded2d trait.
let aabb = hexagon.aabb_2d(translation, rotation);
let circle = hexagon.bounding_circle(translation, rotation);
```

#### Ray Casting and Volume Casting [#](https://bevy.org/news/bevy-0-13/#ray-casting-and-volume-casting)

The bounding volumes also support basic ray casting and volume casting. Ray casting tests if a bounding volume intersects with a given ray, cast from an origin in a direction, until a maximum distance. Volume casts work similarly, but function as if moving a volume along the ray.

This functionality is provided through the new [`RayCast2d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.RayCast2d.html), [`RayCast3d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.RayCast3d.html), [`AabbCast2d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.AabbCast2d.html), [`AabbCast3d`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.AabbCast3d.html), [`BoundingCircleCast`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.BoundingCircleCast.html), and [`BoundingSphereCast`](https://docs.rs/bevy/0.13.0/bevy/math/bounding/struct.BoundingSphereCast.html) types. They can be used to check for intersections against bounding volumes, and to compute the distance from the origin of the cast to the point of intersection.

Below, you can see ray casting, volume casting, and intersection tests in action:

To make it easier to reason about ray casts in different dimensions, the old [`Ray`](https://docs.rs/bevy/0.12.1/bevy/math/struct.Ray.html) type has also been split into [`Ray2d`](https://docs.rs/bevy/0.13.0/bevy/math/struct.Ray2d.html) and [`Ray3d`](https://docs.rs/bevy/0.13.0/bevy/math/struct.Ray3d.html). The new [`Direction2d`](https://docs.rs/bevy/0.13.0/bevy/math/primitives/struct.Direction2d.html) and [`Direction3d`](https://docs.rs/bevy/0.13.0/bevy/math/primitives/struct.Direction3d.html) types are used to ensure that the ray direction remains normalized, providing a type-level guarantee that the vector is always unit-length. These are already in use in some other APIs as well, such as for some primitives and gizmo methods.

## System Stepping [#](https://bevy.org/news/bevy-0-13/#system-stepping)

authors: @dmlary

**Bevy 0.13** adds support for **System Stepping**, which adds debugger-style execution control for systems.

The [`Stepping`](https://docs.rs/bevy/0.13.0/bevy/ecs/schedule/stepping/Stepping.html) resource controls which systems within a schedule execute each frame, and provides step, break, and continue functionality to enable live debugging.

```rust
let mut stepping = Stepping::new();
```

You add the schedules you want to step through to the [`Stepping`](https://docs.rs/bevy/0.13.0/bevy/ecs/schedule/stepping/Stepping.html) resource. The systems in these schedules can be thought of as the "stepping frame". Systems in the "stepping frame" won't run unless a relevant step or continue action occurs. Schedules that are not added will run on every update, even while stepping. This enables core functionality like rendering to continue working.

```rust
stepping.add_schedule(Update);
stepping.add_schedule(FixedUpdate);
```

Stepping is disabled by default, even when the resource is inserted. To enable it in apps, feature flags, dev consoles and obscure hotkeys all work great.

```rust
#[cfg(feature = "my_stepping_flag")]
stepping.enable();
```

Finally, you add the [`Stepping`](https://docs.rs/bevy/0.13.0/bevy/ecs/schedule/stepping/Stepping.html) resource to the ECS [`World`](https://docs.rs/bevy/0.13.0/bevy/ecs/world/struct.World.html).

```rust
app.insert_resource(stepping);
```

### System Step & Continue Frame [#](https://bevy.org/news/bevy-0-13/#system-step-continue-frame)

The "step frame" action runs the system at the stepping cursor, and advances the cursor during the next render frame. This is useful to see individual changes made by systems, and see the state of the world prior to executing a system

```rust
stepping.step_frame()
```

The "continue frame" action will execute systems starting from the stepping cursor to the end of the stepping frame during the next frame. It may stop before the end of the stepping frame if it encounters a system with a breakpoint. This is useful for advancing quickly through an entire frame, getting to the start of the next frame, or in combination with breakpoints.

```rust
stepping.continue_frame()
```

This video demonstrates these actions on the breakout example with a custom `egui` interface. The stepping cursor can be seen moving through the systems list as we click the `step` button. When the `continue` button is clicked, you can see the game progress one stepping frame for each click.

### Breakpoints [#](https://bevy.org/news/bevy-0-13/#breakpoints)

When a schedule grows to a certain point, it can take a long time to step through every system in the schedule just to see the effects of a few systems. In this case, stepping provides system breakpoints.

This video illustrates how a breakpoint on `check_for_collisions()` behaves with "step" and "continue" actions:

### Disabling Systems During Stepping [#](https://bevy.org/news/bevy-0-13/#disabling-systems-during-stepping)

During debugging, it can be helpful to disable systems to narrow down the source of the problem. `Stepping::never_run()` and `Stepping::never_run_node()` can be used to disable systems while stepping is enabled.

### Excluding Systems from Stepping [#](https://bevy.org/news/bevy-0-13/#excluding-systems-from-stepping)

It may be necessary to ensure some systems still run while stepping is enabled. While best-practice is to have them in a schedule that has not been added to the `Stepping` resource, it is possible to configure systems to always run while stepping is enabled. This is primarily useful for event & input handling systems.

Systems can be configured to always run by calling `Stepping::always_run()`, or `Stepping::always_run_node()`. When a system is configured to always run, it will run each rendering frame even when stepping is enabled.

### Limitations [#](https://bevy.org/news/bevy-0-13/#limitations)

There are some limitations in this initial implementation of stepping:

- **Systems that reads events likely will not step properly**: Because frames still advance normally while stepping is enabled, events can be cleared before a stepped system can read them. The best approach here is to configure event-based systems to always run, or put them in a schedule not added to `Stepping`. "Continue" with breakpoints may also work in this scenario.
- **Conditional systems may not run as expected when stepping**: Similar to event-based systems, if the run condition is true for only a short time, system may not run when stepped.

### Detailed Examples [#](https://bevy.org/news/bevy-0-13/#detailed-examples)

- [Text-based stepping example](https://github.com/bevyengine/bevy/blob/main/examples/ecs/system_stepping.rs)
- Non-interactive [bevy UI example stepping plugin](https://github.com/bevyengine/bevy/blob/main/examples/games/stepping.rs) used in the breakout example
- Interactive [egui stepping plugin](https://gist.github.com/dmlary/3fd57ebf1f88bb9afa8a6604737dac97) used in demo videos

## Camera Exposure [#](https://bevy.org/news/bevy-0-13/#camera-exposure)

authors: @superdump (Rob Swain), @JMS55, @cart

In the real world, the brightness of an image captured by a camera is determined by its exposure: the amount of light that the camera's sensor or film incorporates. This is controlled by several mechanics of the camera:

- **Aperture**: Measured in F-Stops, the aperture opens and closes to control how much light is allowed into the camera's sensor or film by physically blocking off lights from specific angles, similar to the pupil of an eye.
- **Shutter Speed**: How long the camera's shutter is open, which is the duration of time that the camera's sensor or film is exposed to light.
- **ISO Sensitivity**: How sensitive the camera's sensor or film is to light. A higher value indicates a higher sensitivity to light.

Each of these plays a role in how much light the final image receives. They can be combined into a final EV number (exposure value), such as the semi-standard EV100 (the exposure value for ISO 100). Higher EV100 numbers mean that more light is required to get the same result. For example, a sunny day scene might require an EV100 of about 15, whereas a dimly lit indoor scene might require an EV100 of about 7.

In **Bevy 0.13**, you can now configure the EV100 on a per-camera basis using the new [`Exposure`](https://docs.rs/bevy/0.13.0/bevy/render/camera/struct.Exposure.html) component. You can set it directly using the [`Exposure::ev100`](https://docs.rs/bevy/0.13.0/bevy/render/camera/struct.Exposure.html#structfield.ev100) field, or you can use the new [`PhysicalCameraParameters`](https://docs.rs/bevy/0.13.0/bevy/render/camera/struct.PhysicalCameraParameters.html) struct to calculate an ev100 using "real world" camera settings like f-stops, shutter speed, and ISO sensitivity.

This is important because Bevy's "physically based" renderer (PBR) is intentionally grounded in reality. Our goal is for people to be able to use real-world units in their lights and materials and have them behave as close to reality as possible.

Drag this image to compare

![EV100 9.7](https://bevy.org/news/bevy-0-13/exposure_97.jpg)![EV100 15](https://bevy.org/news/bevy-0-13/exposure_15.jpg)

Note that prior versions of Bevy hard-coded a static EV100 for some of its light types. In **Bevy 0.13** it is configurable _and_ consistent across all light types. We have also bumped the default EV100 to 9.7, which is a [number we chose to best match Blender's default exposure](https://github.com/bevyengine/bevy/issues/11577#issuecomment-1942873507). It also happens to be a nice "middle ground" value that sits somewhere between indoor lighting and overcast outdoor lighting.

You may notice that point lights now require _significantly_ higher intensity values (in lumens). This (sometimes) million-lumen values might feel exorbitant. Just reassure yourself that (1) it actually requires a lot of light to meaningfully register in an overcast outdoor environment and (2) Blender exports lights on these scales (and we are calibrated to be as close as possible to them).

## Camera-Driven UI [#](https://bevy.org/news/bevy-0-13/#camera-driven-ui)

authors: @bardt, @oceantume

Historically, Bevy's UI elements have been scaled and positioned in the context of the primary window, regardless of the camera settings. This approach made some UI experiences like split-screen multiplayer difficult to implement, and others such as having UI in multiple windows impossible.

**Bevy 0.13** introduces **Camera-Driven UI**. Each camera can now have its own UI root, rendering according to its viewport, scale factor, and a target which can be a secondary window or even a texture.

This change unlocks a variety of new UI experiences, including split-screen multiplayer, UI in multiple windows, displaying non-interactive UI in a 3D world, and more.

![Split-screen with independent UI roots](https://bevy.org/news/bevy-0-13/split-screen.png)

If there is one camera in the world, you don't need to do anything; your UI will be displayed in that camera's viewport.

```rust
commands.spawn(Camera3dBundle {
    // Camera can have custom viewport, target, etc.
});
commands.spawn(NodeBundle {
    // UI will be rendered to the singular camera's viewport
});
```

When more control is desirable, or there are multiple cameras, we introduce the [`TargetCamera`](https://docs.rs/bevy/0.13.0/bevy/ui/struct.TargetCamera.html) component. This component can be added to a root UI node to specify which camera it should be rendered to.

```rust
// For split-screen multiplayer, we set up 2 cameras and 2 UI roots
let left_camera = commands.spawn(Camera3dBundle {
    // Viewport is set to left half of the screen
}).id();

commands
    .spawn((
        TargetCamera(left_camera),
        NodeBundle {
            //...
        }
    ));

let right_camera = commands.spawn(Camera3dBundle {
    // Viewport is set to right half of the screen
}).id();

commands
    .spawn((
        TargetCamera(right_camera),
        NodeBundle {
            //...
        })
    );
```

With this change, we also removed the [`UiCameraConfig`](https://docs.rs/bevy/0.12.1/bevy/ui/camera_config/struct.UiCameraConfig.html) component. If you were using it to hide UI nodes, you can achieve the same outcome by configuring a [`Visibility`](https://docs.rs/bevy/0.13.0/bevy/render/view/enum.Visibility.html) component on the root node.

```rust
commands.spawn(Camera3dBundle::default());
commands.spawn(NodeBundle {
    visibility: Visibility::Hidden, // UI will be hidden
    // ...
});
```

## Texture Slicing and Tiling [#](https://bevy.org/news/bevy-0-13/#texture-slicing-and-tiling)

authors: @ManevilleF

3D rendering gets a lot of love, but 2D features matter too! We're pleased to add CPU-based _slicing and tiling_ to both `bevy_sprite` and `bevy_ui` in **Bevy 0.13**!

This behavior is controlled by a new optional component: [`ImageScaleMode`](https://docs.rs/bevy/0.13.0/bevy/prelude/enum.ImageScaleMode.html).

### 9 slicing [#](https://bevy.org/news/bevy-0-13/#9-slicing)

Adding `ImageScaleMode::Sliced` to an entity with a sprite or UI bundle enables [9 slicing](https://en.wikipedia.org/wiki/9-slice_scaling), keeping the image proportions during resizes, avoiding stretching of the texture.

![Stretched Vs Sliced texture](https://bevy.org/news/bevy-0-13/slice_vs_stretched.png)

This is very useful for UI, allowing your pretty textures to look right even as the size of your element changes.

![Sliced Buttons](https://bevy.org/news/bevy-0-13/ui_slice.png)

Border texture by [Kenney](https://kenney.nl/assets/fantasy-ui-borders)

```rust
commands.spawn((
    SpriteSheetBundle::default(),
    ImageScaleMode::Sliced(TextureSlicer {
        // The image borders are 20 pixels in every direction
        border: BorderRect::square(20.0),
        // we don't stretch the corners more than their actual size (20px)
        max_corner_scale: 1.0,
        ..default()
    }),
));
```

### Tiling [#](https://bevy.org/news/bevy-0-13/#tiling)

Adding `ImageMode::Tiled { .. }` to your 2D sprite entities enables _texture tiling_: repeating the image until their entire area is filled. This is commonly used for backgrounds and surfaces.

```rust
commands.spawn((
    SpriteSheetBundle::default(),
    ImageScaleMode::Tiled {
        // The image will repeat horizontally
        tile_x: true,
        // The image will repeat vertically
        tile_y: true,
        // The texture will repeat if the drawing rect is larger than the image size
        stretch_value: 1.0,
    },
));
```

## Dynamic Queries [#](https://bevy.org/news/bevy-0-13/#dynamic-queries)

authors: @james-j-obrien, @jakobhellermann, @Suficio

In Bevy ECS, queries use a type-powered DSL. The full type of the query (what component to access, which filter to use) must be specified at compile time.

Sometimes we can't know what data the query wants to access at compile time. Some scenarios just cannot be done with static queries:

- Defining queries in scripting languages like Lua or JavaScript.
- Defining new components from a scripting language and query them.
- Adding a runtime filter to entity inspectors like [`bevy-inspector-egui`](https://crates.io/crates/bevy-inspector-egui).
- Adding a [Quake-style console](https://github.com/doonv/bevy_dev_console) to modify or query components from a prompt at runtime.
- Creating an [editor with remote capabilities](https://makeshift-bevy-web-editor.vercel.app/).

Dynamic queries make these all possible. And these are only the plans we've heard about so far!

The standard way of defining a [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) is by using them as system parameters:

```rust
fn take_damage(mut player_health: Query<(Entity, &mut Health), With<Player>>) {
    // ...
}
```

**This won't change.** And for most (if not all) gameplay use cases, you will continue to happily use the delightfully simple [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) API.

However, consider this situation: as a game or mod developer I want to list entities with a specific component through a text prompt. Similar to how the Quake console works. What would that look like?

```rust
#[derive(Resource)]
struct UserQuery(String);

// user_query is entered as a text prompt by the user when the game is running.
// In a system, it's quickly apparent that we can't use `Query`.
fn list_entities_system(user_query: Res<UserQuery>, query: Query<FIXME, With<FIXME>>) {}

// Even when using the more advanced `World` API, we are stuck.
fn list_entities(user_query: String, world: &mut World) {
    // FIXME: what type goes here?
    let query = world.query::<FIXME>();
}
```

It's impossible to choose a type based on the value of `user_query`! [`QueryBuilder`](https://docs.rs/bevy/0.13.0/bevy/ecs/prelude/struct.QueryBuilder.html) solves this problem.

```rust
fn list_entities(
    user_query: String,
    type_registry: &TypeRegistry,
    world: &mut World,
) -> Option<()> {
    let name = user_query.split(' ').next()?;
    let type_id = type_registry.get_with_short_type_path(name)?.type_id();
    let component_id = world.components().get_id(type_id)?;

    let query = QueryBuilder::<FilteredEntityRef>::new(&mut world)
        .ref_id(component_id)
        .build();

    for entity_ref in query.iter(world) {
        let ptr = entity_ref.get_by_id(component_id);
        // Convert `ptr` into a `&dyn Reflect` and use it.
    }
    Some(())
}
```

It is still an error-prone, complex, and unsafe API, but it makes something that was previously impossible possible. We expect third-party crates to provide convenient wrappers around the `QueryBuilder` API, some of which will undoubtedly make their way upstream.

## Query Transmutation [#](https://bevy.org/news/bevy-0-13/#query-transmutation)

authors: @hymm, james-j-obrien

Have you ever wanted to pass a query to a function, but instead of having a `Query<&Transform>` you have a `Query<(&Transform, &Velocity), With<Enemy>>`? In **Bevy 0.13** you can, thanks to the new [`QueryLens`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.QueryLens.html) and [`Query::transmute_lens()`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html#method.transmute_lens) method.

Query transmutes allow you to change a query into different query types as long as the components accessed are a subset of the original query. If you do try to access data that is not in the original query, this method will panic.

```rust
fn reusable_function(lens: &mut QueryLens<&Transform>) {
    let query = lens.query();
    // do something with the query...
}

// We can use the function in a system that takes the exact query.
fn system_1(mut query: Query<&Transform>) {
    reusable_function(&mut query.as_query_lens());
}

// We can also use it with a query that does not match exactly
// by transmuting it.
fn system_2(mut query: Query<(&mut Transform, &Velocity), With<Enemy>>) {
    let mut lens = query.transmute_lens::<&Transform>();
    reusable_function(&mut lens);
}
```

Note that the [`QueryLens`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.QueryLens.html) will still iterate over the same entities as the original [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) it is derived from. A `QueryLens<&Transform>` taken from a `Query<(&Transform, &Velocity)>`, will only include the `Transform` of entities with both `Transform` and `Velocity` components.

Besides removing parameters you can also change them in limited ways to the different smart pointer types. One of the more useful is to change a `&mut` to a `&`. See the [documentation](https://docs.rs/bevy/latest/bevy/ecs/system/struct.Query.html#method.transmute_lens) for more details.

One thing to take into consideration is the transmutation is not free. It works by creating a new state and copying cached data inside the original query. It's not an expensive operation, but you should avoid doing it inside a hot loop.

## `WorldQuery` Trait Split [#](https://bevy.org/news/bevy-0-13/#worldquery-trait-split)

authors: @wainwrightmark @taizu-jin

A [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) has two type parameters: one for the data to be fetched, and a second optional one for the filters.

In previous versions of Bevy both parameters simply required [`WorldQuery`](https://docs.rs/bevy/0.12.0/bevy/ecs/query/trait.WorldQuery.html): there was nothing stopping you from using types intended as filters in the data position (or vice versa).

Apart from making the type signature of the [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) items more complicated (see example below) this usually worked fine as most filters had the same behavior in either position.

Unfortunately this was not the case for [`Changed`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/struct.Changed.html) and [`Added`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/struct.Added.html) which had different (and undocumented) behavior in the data position and this could lead to bugs in user code.

To allow us to prevent this type of bug at compile time, the [`WorldQuery`](https://docs.rs/bevy/0.12.0/bevy/ecs/query/trait.WorldQuery.html) trait has been replaced by two traits: [`QueryData`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/trait.QueryData.html) and [`QueryFilter`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/trait.QueryFilter.html). The data parameter of a [`Query`](https://docs.rs/bevy/0.13.0/bevy/ecs/system/struct.Query.html) must now be [`QueryData`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/trait.QueryData.html) and the filter parameter must be [`QueryFilter`](https://docs.rs/bevy/0.13.0/bevy/ecs/query/trait.QueryFilter.html).

Most user code should be unaffected or easy to migrate.

```rust
// Probably a subtle bug: `With` filter in the data position - will not compile in 0.13
fn my_system(query: Query<(Entity, With<ComponentA>)>)
{
    // The type signature of the query items is `(Entity, ())`, which is usable but unwieldy
  for (entity, _) in query.iter(){
  }
}

// Idiomatic, compiles in both 0.12 and 0.13
fn my_system(query: Query<Entity, With<ComponentA>>)
{
  for entity in query.iter(){
  }
}
```

## Automatically Insert `apply_deferred` Systems [#](https://bevy.org/news/bevy-0-13/#automatically-insert-apply-deferred-systems)

authors: @hymm

When writing gameplay code, you might commonly have one system that wants to immediately see the effects of commands queued in another system. Before **Bevy 0.13**, you would have to manually insert an `apply_deferred` system between the two, a special system which causes those commands to be applied when encountered. Bevy now detects when a system with commands is ordered relative to other systems and inserts the `apply_deferred` for you.

```rust
// Before 0.13
app.add_systems(
    Update,
    (
        system_with_commands,
        apply_deferred,
        another_system,
    ).chain()
);
```

```rust
// After 0.13
app.add_systems(
    Update,
    (
        system_with_commands,
        another_system,
    ).chain()
);
```

This resolves a common beginner footgun: if two systems are ordered, shouldn't the second always see the results of the first?

Automatically inserted `apply_deferred` systems are optimized by automatically merging them if possible. In most cases, it is recommended to remove all manually inserted `apply_deferred` systems, as allowing Bevy to insert and merge these systems as needed will usually be both faster and involve less boilerplate.

```rust
// This will only add one apply_deferred system.
app.add_systems(
    Update,
    (
        (system_1_with_commands, system_2).chain(),
        (system_3_with_commands, system_4).chain(),
    )
);
```

If this new behavior does not work for you, please consult the migration guide. There are several new APIs that allow you to opt-out.

## More Flexible One-Shot Systems [#](https://bevy.org/news/bevy-0-13/#more-flexible-one-shot-systems)

authors: @Nathan-Fenner

In **Bevy 0.12**, we introduced [one-shot systems](https://bevy.org/news/bevy-0-12/#one-shot-systems), a handy way to call systems on demand without having to add them to a schedule. The initial implementation had some limitations in regard to what systems could and could not be used as one-shot systems. In **Bevy 0.13**, these limitations have been resolved.

One-shot systems now support inputs and outputs.

```rust
fn increment_sys(In(increment_by): In<i32>, mut counter: ResMut<Counter>) -> i32 {
    counter.0 += increment_by;
    counter.0
}

let mut world = World::new();
let id = world.register_system(increment_sys);

world.insert_resource(Counter(1));
let count_one = world.run_system_with_input(id, 5).unwrap(); // increment counter by 5 and return 6
let count_two = world.run_system_with_input(id, 2).unwrap(); // increment counter by 2 and return 8
```

Running a system now returns the system output as `Ok(output)`. Note that output cannot be returned when calling one-shot systems through commands, because of their deferred nature.

Exclusive systems can now be registered as one-shot systems:

```rust
world.register_system(|world: &mut World| { /* do anything */ });
```

Boxed systems can now be registered with `register_boxed_system`.

These improvements round out one-shot systems significantly: they should now work just like any other Bevy system.

## wgpu 0.19 Upgrade and Rendering Performance Improvements [#](https://bevy.org/news/bevy-0-13/#wgpu-0-19-upgrade-and-rendering-performance-improvements)

authors: @Elabajaba, @JMS55

In **Bevy 0.13** we upgraded from `wgpu` 0.17 to `wgpu` 0.19, which includes the long awaited `wgpu` [arcanization](https://gfx-rs.github.io/2023/11/24/arcanization.html) that allows us to [compile shaders asynchronously](https://github.com/bevyengine/bevy/pull/10812) to avoid shader compilation stutters and [multithread draw call creation](https://github.com/bevyengine/bevy/pull/9172) for better performance in CPU-bound scenes.

Due to changes in wgpu 0.19, we've added a new `webgpu` feature to Bevy that is now required when doing WebAssembly builds targeting WebGPU. Disabling the `webgl2` feature is no longer required when targeting WebGPU, but the new `webgpu` feature currently overrides the `webgl2` feature when enabled. Library authors, please do not enable the `webgpu` feature by default. In the future we plan on allowing you to target both WebGL2 and WebGPU in the same WebAssembly binary, but we aren't quite there yet.

We've swapped the material and mesh bind groups, so that mesh data is now in bind group 1, and material data is in bind group 2. This greatly improved our draw call batching when combined with changing the sorting functions for the opaque passes to sort by pipeline and mesh. Previously we were sorting them by distance from the camera. These batching improvements mean we're doing fewer draw calls, which improves CPU performance, especially in larger scenes. We've also removed the `get_instance_index` function in shaders, as it was only required to work around an upstream bug that has been fixed in wgpu 0.19. For other shader or rendering changes, please see the [migration guide](https://bevy.org/learn/migration-guides/0-12-to-0-13/) and [wgpu's changelog](https://github.com/gfx-rs/wgpu/blob/v0.19/CHANGELOG.md).

Many small changes both to Bevy and `wgpu` summed up to make a modest but measurable difference in our performance on realistic 3D scenes! We ran some quick tests on both **Bevy 0.12** and **Bevy 0.13** on the same machine on four complex scenes: [Bistro](https://github.com/DGriffin91/bevy_bistro_scene), [Sponza](https://github.com/DGriffin91/bevy_sponza_scene), [San Miguel](https://github.com/DGriffin91/bevy_san_miguel_scene) and [Hidden Alley](https://blog.polyhaven.com/hidden-alley/).

![A high polygon, realistically lit screenshot of a beautiful cafe with a tree in the background.](https://bevy.org/news/bevy-0-13/San_Miguel_13.jpg)

As you can see, these scenes are substantially more detailed than most video game environments, but that screenshot was being rendered in Bevy at better than 60 FPS at 1440p resolution! Between Bevy 0.12 and Bevy 0.13 we saw frame times decrease by about 5-10% across the scenes tested. Nice work!

![A graph showing the FPS of the different scenes. Bistro went from 90 FPS to 120 FPS, while the other scenes improved slightly. All scenes were 60-120 FPS.](https://bevy.org/news/bevy-0-13/rendering-perf-graph.svg)

## Unload Rendering Assets from RAM [#](https://bevy.org/news/bevy-0-13/#unload-rendering-assets-from-ram)

authors: @JMS55, @mockersf, @brianreavis

Meshes and the textures used to define their materials take up a ton of memory: in many games, memory usage is the biggest limitation on the resolution and polygon count of the game! Moreover, transferring that data from system RAM (used by the CPU) to the VRAM (used by the GPU) can be a real performance bottleneck.

**Bevy 0.13** adds the ability to unload this data from system RAM, once it has been successfully transferred to VRAM. To configure this behavior for your asset, set the [`RenderAssetUsages`](https://docs.rs/bevy/0.13.0/bevy/render/render_asset/struct.RenderAssetUsages.html) field to specify whether to retain the data in the main (CPU) world, the render (GPU) world, or both.

This behavior is currently off by default for most asset types as it [has some caveats](https://github.com/bevyengine/bevy/pull/11212) (given that the asset becomes unavailable to logic on the CPU), but we strongly recommend enabling it for your assets whenever possible for significant memory usage wins (and we will likely enable it by default in the future).

Texture atlases and font atlases now only extract data that's actually in use to VRAM, rather than wasting work sending _all_ possible images or characters to VRAM every frame. Neat!

## Better Batching Through Smarter Sorting [#](https://bevy.org/news/bevy-0-13/#better-batching-through-smarter-sorting)

authors: @Elabajaba

One of the core techniques used to speed up rendering is to draw many similar objects together at the same time. In this case, Bevy was already using a technique called "batching", which allows us to combine multiple similar operations, reducing the number of expensive draw calls (instructions to the GPU) that are being made.

However, our strategy for defining these batches was far from optimal. Previously, we were sorting by distance to the camera, and _then_ checking if multiple of the same meshes were adjacent to each other in that sorted list. In realistic scenes, this is unlikely to find many candidates for merging!

In **Bevy 0.13**, we first sort by pipeline (effectively the type of material being used), and then by mesh identity. This strategy results in better batching, improving overall FPS by double-digit percentages on the [realistic scene tested](https://syntystore.com/products/polygon-fantasy-kingdom)!

![A graph showing batching improvements. Shadows are very expensive, and FPS improved by at least 20% in all cases tested.](https://bevy.org/news/bevy-0-13/better_batching.svg)

## Animation Interpolation Methods [#](https://bevy.org/news/bevy-0-13/#animation-interpolation-methods)

authors: @mockersf

Generally, animations are defined by their **keyframes**: snapshots of the position (and other state) or objects at moments along a timeline. But what happens between those keyframes? Game engines need to **interpolate** between them, smoothly transitioning from one state to the next.

The simplest interpolation method is linear: the animated object just moves an equal distance towards the next keyframe every unit of time. But this isn't always the desired effect! Both stop-motion-style and more carefully smoothed animations have their place.

Bevy now supports both step and cubic spline interpolation in animations. Most of the time, this will just be parsed correctly from the glTF files, but when setting [`VariableCurve`](https://docs.rs/bevy/0.13.0/bevy/animation/struct.VariableCurve.html) manually, there's a new [`Interpolation`](https://docs.rs/bevy/0.13.0/bevy/animation/enum.Interpolation.html) field to set.

![Demonstrating the different types of interpolation](https://bevy.org/news/bevy-0-13/interpolation_methods.gif)

## `Animatable` Trait [#](https://bevy.org/news/bevy-0-13/#animatable-trait)

authors: @james7132

When you think of "animation": you're probably imagining moving objects through space. Translating them back and forth, rotating them, maybe even squashing and stretching them. But in modern game development, animation is a powerful shared set of tools and concepts for "changing things over time". Transforms are just the beginning: colors, particle effects, opacity and even boolean values like visibility can all be animated!

In **Bevy 0.13**, we've taken the first step towards [this vision](https://github.com/bevyengine/rfcs/blob/main/rfcs/51-animation-composition.md), with the [`Animatable`](https://docs.rs/bevy/0.13.0/bevy/prelude/trait.Animatable.html) trait.

```rust
/// An animatable value type.
pub trait Animatable: Reflect + Sized + Send + Sync + 'static {
    /// Interpolates between `a` and `b` with  a interpolation factor of `time`.
    ///
    /// The `time` parameter here may not be clamped to the range `[0.0, 1.0]`.
    fn interpolate(a: &Self, b: &Self, time: f32) -> Self;

    /// Blends one or more values together.
    ///
    /// Implementors should return a default value when no inputs are provided here.
    fn blend(inputs: impl Iterator<Item = BlendInput<Self>>) -> Self;

    /// Post-processes the value using resources in the [`World`].
    /// Most animatable types do not need to implement this.
    fn post_process(&mut self, _world: &World) {}
}
```

This is the first step towards animation blending and an asset-driven animation graph which is essential for shipping large scale 3D games in Bevy. But for now, this is just a building block. We've implemented this for a few key types (`Transform`, `f32` and `glam`'s `Vec` types) and published the trait. Slot it into your games and crates, and team up with other contributors to help `bevy_animation` become just as pleasant and featureful as the rest of the engine.

## Extensionless Asset Support [#](https://bevy.org/news/bevy-0-13/#extensionless-asset-support)

authors: @bushrat011899

In prior versions of Bevy, the default way to choose an [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html) for a particular asset was entirely based on file extensions. The [recent addition of .meta files](https://bevy.org/news/bevy-0-12/#asset-meta-files) allowed for specifying more granular loading behavior, but file extensions were still required. In **Bevy 0.13**, the asset type can now be used to infer the [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html).

```rust
// Uses AudioSettingsAssetLoader
let audio = asset_server.load("data/audio.json");

// Uses GraphicsSettingsAssetLoader
let graphics = asset_server.load("data/graphics.json");
```

This is possible because every [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html) is required to declare what **type** of asset it loads, not just the extensions it supports. Since the [`load`](https://docs.rs/bevy/0.13.0/bevy/asset/struct.AssetServer.html#method.load) method on [`AssetServer`](https://docs.rs/bevy/0.13.0/bevy/asset/struct.AssetServer.html) was already generic over the type of asset to return, this information is already available to the [`AssetServer`](https://docs.rs/bevy/0.13.0/bevy/asset/struct.AssetServer.html).

```rust
// The above example with types shown
let audio: Handle<AudioSettings> = asset_server.load::<AudioSettings>("data/audio.json");
let graphics: Handle<GraphicsSettings> = asset_server.load::<GraphicsSettings>("data/graphics.json");
```

Now we can also use it to choose the [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html) itself.

When loading an asset, the loader is chosen by checking (in order):

1. The asset `meta` file
2. The type of `Handle<A>` to return
3. The file extension

```rust
// This will be inferred from context to be a glTF asset, ignoring the file extension
let gltf_handle = asset_server.load("models/cube/cube.gltf");

// This still relies on file extension due to the label
let cube_handle = asset_server.load("models/cube/cube.gltf#Mesh0/Primitive0");
//                                                        ^^^^^^^^^^^^^^^^^
//                                                        | Asset path label
```

### File Extensions Are Now Optional [#](https://bevy.org/news/bevy-0-13/#file-extensions-are-now-optional)

Since the asset type can be used to infer the loader, neither the file to be loaded nor the [`AssetLoader`](https://docs.rs/bevy/0.13.0/bevy/asset/trait.AssetLoader.html) need to have file extensions.

```rust
pub trait AssetLoader: Send + Sync + 'static {
    /* snip */

    /// Returns a list of extensions supported by this [`AssetLoader`], without the preceding dot.
    fn extensions(&self) -> &[&str] {
        // A default implementation is now provided
        &[]
    }
}
```

Previously, an asset loader with no extensions was very cumbersome to use. Now, they can be used just as easily as any other loader. Likewise, if a file is missing its extension, Bevy can now choose the appropriate loader.

```rust
let license = asset_server.load::<Text>("LICENSE");
```

Appropriate file extensions are still recommended for good project management, but this is now a recommendation rather than a hard requirement.

### Multiple Asset Loaders With The Same Asset [#](https://bevy.org/news/bevy-0-13/#multiple-asset-loaders-with-the-same-asset)

Now, a single path can be used by multiple asset handles as long as they are distinct asset types.

```rust
// Load the sound effect for playback
let bang = asset_server.load::<AudioSource>("sound/bang.ogg");

// Load the raw bytes of the same sound effect (e.g, to send over the network)
let bang_blob = asset_server.load::<Blob>("sound/bang.ogg");

// Returns the bang handle since it was already loaded
let bang_again = asset_server.load::<AudioSource>("sound/bang.ogg");
```

Note that the above example uses [turbofish](https://turbo.fish/) syntax for clarity. In practice, it's not required, since the type of asset loaded can usually be inferred at the call site.

```rust
#[derive(Resource)]
struct SoundEffects {
    bang: Handle<AudioSource>,
    bang_blob: Handle<Blob>,
}

fn setup(mut effects: ResMut<SoundEffects>, asset_server: Res<AssetServer>) {
    effects.bang = asset_server.load("sound/bang.ogg");
    effects.bang_blob = asset_server.load("sound/bang.ogg");
}
```

The [`custom_asset` example](https://bevy.org/examples/Assets/custom-asset/) has been updated to demonstrate these new features.

## Texture Atlas Rework [#](https://bevy.org/news/bevy-0-13/#texture-atlas-rework)

authors: @ManevilleF

Texture atlases efficiently combine multiple images into a single larger texture called an atlas.

**Bevy 0.13** significantly reworks them to reduce boilerplate and make them more data-oriented. Say goodbye to `TextureAtlasSprite` and `UiTextureAtlasImage` components (and their corresponding `Bundle` types). Texture atlasing is now enabled by adding a single _additional_ component to normal sprite and image entities: [`TextureAtlas`](https://docs.rs/bevy/0.13.0/bevy/sprite/struct.TextureAtlas.html).

### Why? [#](https://bevy.org/news/bevy-0-13/#why)

Texture atlases (sometimes called sprite sheets) simply draw a custom _section_ of a given texture. This is _still_ Sprite-like or Image-like behavior, we're just drawing a subset. The new [`TextureAtlas`](https://docs.rs/bevy/0.13.0/bevy/sprite/struct.TextureAtlas.html) component embraces that by storing:

- a `Handle<TextureAtlasLayout>`, an asset mapping an index to a `Rect` section of a texture
- a `usize` index defining which section `Rect` of the layout we want to display

## Light `RenderLayers` [#](https://bevy.org/news/bevy-0-13/#light-renderlayers)

authors: @robftm

[`RenderLayers`](https://docs.rs/bevy/latest/bevy/render/view/struct.RenderLayers.html) are Bevy's answer to quickly hiding and showing entities en masse by filtering what a Camera can see ... great for things like customizing the first-person view of what a character is holding (or making sure vampires don't show up in your mirrors!).

[`RenderLayers`](https://docs.rs/bevy/latest/bevy/render/view/struct.RenderLayers.html) [now play nice](https://github.com/bevyengine/bevy/pull/10742) with lights, fixing a serious limitation to make sure this awesome feature can shine appropriately!

## Bind Group Layout Entries [#](https://bevy.org/news/bevy-0-13/#bind-group-layout-entries)

authors: @IceSentry

We added a new API, inspired by the bind group entries API from 0.12, to declare bind group layouts. This new API is based on using built-in functions to define bind group layout resources and automatically set the index based on its position.

Here's a short example of how declaring a new layout looks:

```rust
let layout = render_device.create_bind_group_layout(
    "post_process_bind_group_layout",
    &BindGroupLayoutEntries::sequential(
        ShaderStages::FRAGMENT,
        (
            texture_2d_f32(),
            sampler(SamplerBindingType::Filtering),
            uniform_buffer::<PostProcessingSettings>(false),
        ),
    ),
);
```

## Type-Safe Labels for the `RenderGraph` [#](https://bevy.org/news/bevy-0-13/#type-safe-labels-for-the-rendergraph)

authors: @DasLixou

Bevy uses Rust's type system extensively when defining labels, letting developers lean on tooling to catch typos and ease refactors. But this didn't apply to Bevy's render graph. In the render graph, hard-coded—and potentially overlapping—strings were used to define nodes and sub-graphs.

```rust
// Before 0.13
impl MyRenderNode {
    pub const NAME: &'static str = "my_render_node"
}
```

In **Bevy 0.13**, we're using a more robust way to name render nodes and render graphs with the help of the type-safe label pattern already used by `bevy_ecs`.

```rust
// After 0.13
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct PrettyFeature;
```

With those, the long paths for const-values become shorter and cleaner:

```rust
// Before 0.13
render_app
    .add_render_graph_node::<ViewNodeRunner<PrettyFeatureNode>>(
        core_3d::graph::NAME,
        PrettyFeatureNode::NAME,
    )
    .add_render_graph_edges(
        core_3d::graph::NAME,
        &[
            core_3d::graph::node::TONEMAPPING,
            PrettyFeatureNode::NAME,
            core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
        ],
    );

// After 0.13
use bevy::core_pipeline::core_3d::graph::{Node3d, Core3d};

render_app
    .add_render_graph_node::<ViewNodeRunner<PrettyFeatureNode>>(
        Core3d,
        PrettyFeature,
    )
    .add_render_graph_edges(
        Core3d,
        (
            Node3d::Tonemapping,
            PrettyFeature,
            Node3d::EndMainPassPostProcessing,
        ),
    );
```

When you need dynamic labels for render nodes, those can still be achieved via e.g. tuple structs:

```rust
#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct MyDynamicLabel(&'static str);
```

This is particularly nice because we don't have to store strings here: we can use integers, custom enums or any other hashable type.

## Winit Upgrade [#](https://bevy.org/news/bevy-0-13/#winit-upgrade)

authors: @Thierry Berger, @mockersf

Through the heroic efforts of our contributors and reviewers, Bevy is [now upgraded](https://github.com/bevyengine/bevy/pull/10702) to use `winit 0.29`. [`winit`](https://docs.rs/winit/latest/winit/) is our windowing library: it abstracts over all the different operating systems and input devices that end users might have, and provides a uniform API to enable a write-once run-anywhere experience. While this brings with it the usual litany of valuable [bug fixes and stability improvements](https://github.com/rust-windowing/winit/blob/master/CHANGELOG.md#0292), the critical change revolves around how [`KeyCode`](https://docs.rs/bevy/latest/bevy/input/keyboard/enum.KeyCode.html) is handled.

Previously, [`KeyCode`](https://docs.rs/bevy/latest/bevy/input/keyboard/enum.KeyCode.html) represented the logical meaning of a key on a keyboard: pressing the same button on the same keyboard when swapping between QWERTY and AZERTY keyboard layouts would give a different result! Now, [`KeyCode`](https://docs.rs/bevy/latest/bevy/input/keyboard/enum.KeyCode.html) represents the physical location of the key. Lovers of WASD games know that this is a much better default for games. For most Bevy developers, you can leave your existing code untouched and simply benefit from better default keybindings for users on non-QWERTY keyboards or layouts. If you need information about the logical keys pressed, use the [`ReceivedCharacter`](https://docs.rs/bevy/latest/bevy/prelude/struct.ReceivedCharacter.html) event.

## Multiple Gizmo Configurations [#](https://bevy.org/news/bevy-0-13/#multiple-gizmo-configurations)

authors: @jeliag

Gizmos let you quickly draw shapes using an immediate mode API. Here is how you use them:

```rust
// Bevy 0.12.1
fn set_gizmo_width(mut config: ResMut<GizmoConfig>) {
    // set the line width of every gizmos with this global configuration resource.
    config.line_width = 5.0;
}

fn draw_circles(mut gizmos: Gizmos) {
    // Draw two circles with a 5 pixels outline
    gizmos.circle_2d(vec2(100., 0.), 120., Color::NAVY);
    gizmos.circle_2d(vec2(-100., 0.), 120., Color::ORANGE);
}
```

Add a [`Gizmos`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/struct.Gizmos.html) system param and simply call a few methods. Cool!

Gizmos are also great for crate authors, they can use the same API. For example, the [`oxidized_navigation`](https://crates.io/crates/oxidized_navigation) navmesh library uses gizmos for its debug overlay. Neat!

However, there is only one global configuration. Therefore, a dependency could very well affect the game's gizmos. It could even make them completely unusable.

Not so great. How to solve this? Gizmo groups.

Now, [`Gizmos`](https://docs.rs/bevy/0.13.0/bevy/gizmos/prelude/struct.Gizmos.html) comes with an optional parameter. By default, it uses a global configuration:

```rust
fn draw_circles(mut default_gizmos: Gizmos) {
    default_gizmos.circle_2d(vec2(100., 0.), 120., Color::NAVY);
}
```

But with a [`GizmoConfigGroup`](https://docs.rs/bevy/0.13.0/bevy/gizmos/config/trait.GizmoConfigGroup.html) parameter, `Gizmos` can choose a distinct configuration:

```rust
fn draw_circles(
    mut default_gizmos: Gizmos,
    // this uses the distinct NavigationGroup config
    mut navigation_gizmos: Gizmos<NavigationGroup>,
) {
    // Two circles with different outline width
    default_gizmos.circle_2d(vec2(100., 0.), 120., Color::NAVY);
    navigation_gizmos.circle_2d(vec2(-100., 0.), 120., Color::ORANGE);
}
```

Create your own gizmo config group by deriving `GizmoConfigGroup`, and registering it to the `App`:

```rust
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct NavigationGroup;

impl Plugin for NavigationPlugin {
    fn build(&mut self, app: &mut App) {
        app
            .init_gizmo_group::<NavigationGroup>()
            // ... rest of plugin initialization.
    }
}
```

And this is how you set the configuration of gizmo groups to different values:

```rust
// Bevy 0.13.0
set_gizmo_width(mut config_store: ResMut<GizmoConfigStore>) {
    let config = config_store.config_mut::<DefaultGizmoConfigGroup>().0;
    config.line_width = 20.0;

    let navigation_config = config_store.config_mut::<NavigationGroup>().0;
    navigation_config.line_width = 10.0;
}
```

Now, the navigation gizmos have a fully separate configuration and don't conflict with the game's gizmos.

Not only that, but the game dev can integrate and toggle the navigation gizmos with their own debug tools however they wish. Be it a hotkey, a debug overlay UI button, or an RPC call. The world is your oyster.

## glTF Extensions [#](https://bevy.org/news/bevy-0-13/#gltf-extensions)

authors: @CorneliusCornbread

**[glTF](https://www.khronos.org/gltf/)** is a popular standardized open file format, used to store and share 3D models and scenes between different programs. The trouble with standards though is that you eventually want to _customize_ it, just a little, to better meet your needs. Khronos Group, in their wisdom, foresaw this and defined a standardized way to customize the format called **[extensions](https://kcoley.github.io/glTF/extensions/)**.

Extensions can be readily exported from other tools (like Blender), and contain [all sorts](https://github.com/KhronosGroup/glTF/blob/main/extensions/README.md) of other useful information: from bleeding edge physically-based material information like anisotropy to performance hints like how to instance meshes.

Because Bevy parses loaded glTF's into our own entity-based hierarchy of objects, getting access to this information when you want to do new rendering things can be hard! With [the changes by CorneliusCornbread](https://github.com/bevyengine/bevy/pull/11138) you can configure the loader to store a raw copy of the glTF file itself with your loaded asset, allowing you to parse and reprocess this information however you please.

## Asset Transformers [#](https://bevy.org/news/bevy-0-13/#asset-transformers)

authors: @thepackett, @RyanSpaker

Asset processing, at its core, involves implementing the `Process` trait, which takes some byte data representing an asset, transforms it, and then returns the processed byte data. However, implementing the `Process` trait by hand is somewhat involved, and so a generic `LoadAndSave<L: AssetLoader, S: AssetSaver>` `Process` implementation was written to make asset processing more ergonomic.

Using the `LoadAndSave` `Process` implementation, the previous Asset processing pipeline had four stages:

1. An `AssetReader` reads some asset source (filesystem, http, etc) and gets the byte data of an asset.
2. An `AssetLoader` reads the byte data and converts it to a Bevy `Asset`.
3. An `AssetSaver` takes a Bevy `Asset`, processes it, and then converts it back into byte data.
4. An `AssetWriter` then writes the asset byte data back to the asset source.

`AssetSaver`s were responsible for both transforming an asset and converting it into byte data. However, this posed a bit of an issue for code reusability. Every time you wanted to transform some asset, such as an image, you would need to rewrite the portion that converts the asset to byte data. To solve this, `AssetSaver`s are now solely responsible for converting an asset into byte data, and `AssetTransformer`s which are responsible for transforming an asset were introduced. A new `LoadTransformAndSave<L: AssetLoader, T: AssetTransformer, S: AssetSaver>` `Process` implementation was added to utilize the new `AssetTransformer`s.

The new asset processing pipeline, using the `LoadTransformAndSave` `Process` implementation, has five stages:

1. An `AssetReader` reads some asset source (filesystem, http, etc) and gets the byte data of an asset.
2. An `AssetLoader` reads the byte data and converts it to a Bevy `Asset`.
3. An `AssetTransformer` takes an asset and transforms it in some way.
4. An `AssetSaver` takes a Bevy `Asset` and converts it back into byte data.
5. An `AssetWriter` then writes the asset byte data back to the asset source.

In addition to having better code reusability, this change encourages writing `AssetSaver`s for various common asset types, which could be used to add runtime asset saving functionality to the `AssetServer`.

The previous `LoadAndSave` `Process` implementation still exists, as there are some cases where an asset transformation step is unnecessary, such as when saving assets into a compressed format.

See the [Asset Processing Example](https://github.com/bevyengine/bevy/blob/main/examples/asset/processing/asset_processing.rs) for a more detailed look into how to use `LoadTransformAndSave` to process a custom asset.

## Entity Optimizations [#](https://bevy.org/news/bevy-0-13/#entity-optimizations)

authors: @Bluefinger, @notverymoe, @scottmcm, @james7132, @NathanSWard

`Entity` (Bevy's 64-bit unique identifier for entities) received several changes this cycle, laying some more groundwork for relations alongside _related_, and nice to have, performance optimizations. The work here involved a lot of deep-diving into compiler codegen/assembly output, with running lots of benchmarks and testing to ensure all changes didn't cause breakages or major problems. Although the work here was dealing with mostly _safe_ code, there were lots of underlying assumptions being changed that could have impacted code elsewhere. This was the most "micro-optimization" oriented set of changes in Bevy 0.13.

- [#9797](https://github.com/bevyengine/bevy/pull/9797): created a unified identifier type, paving the path for us to use the same fast, complex code in both our `Entity` type and the much-awaited relations
- [#9907](https://github.com/bevyengine/bevy/pull/9907): allowed us to store `Option<Entity>` in the same number of bits as `Entity`, by changing the layout of our Entity type to reserve exactly one `u64` value for the `None` variant
- [#10519](https://github.com/bevyengine/bevy/pull/10519): swapped us to a manually crafted `PartialEq` and `Hash` implementation for `Entity` to improve speed and save instructions in our hot loops
- [#10558](https://github.com/bevyengine/bevy/pull/10558): combined the approach of [#9907](https://github.com/bevyengine/bevy/pull/9907) and [#10519](https://github.com/bevyengine/bevy/pull/10519) to optimize `Entity`'s layout further, and optimized our `PartialOrd` and `Ord` implementations!
- [#10648](https://github.com/bevyengine/bevy/pull/10648): further optimized our entity hashing, changing how we multiply in the hash to save one precious assembly instruction in the optimized compiler output

Full credit is also due to the authors who pursued similar work in [#2372](https://github.com/bevyengine/bevy/pull/2372) and [#3788](https://github.com/bevyengine/bevy/pull/3788): while their work was not ultimately merged, it was an incredibly valuable inspiration and source of prior art to base these more recent changes on.

![Benchmark results of optimization work](https://bevy.org/news/bevy-0-13/entity_hash_optimsation_benches.png)

The above results show from where we started (`optimised_eq` being the first PR that introduced the benchmarks) to where we are now with all the optimizations in place (`optimised_entity`). There are improvements across the board, with clear performance benefits that should impact multiple areas of the codebase, not just when hashing entities.

There are a ton of crunchy, well-explained details in the linked PRs, including some fascinating assembly output analysis. If that interests you, open some new tabs in the background!

## Porting `Query::for_each` to `QueryIter::fold` override [#](https://bevy.org/news/bevy-0-13/#porting-query-for-each-to-queryiter-fold-override)

authors: @james7132

Currently to get the full performance out of iterating over queries, `Query::for_each` must be used to take advantage of auto-vectorization and internal iteration optimizations that the compiler can apply. However, this isn't idiomatic rust and is not an iterator method so you can't use it on an iterator chain. However, it is possible to get the same benefits for some iterator methods, for which [#6773](https://github.com/bevyengine/bevy/pull/6773/) by @james7132 sought to achieve. By providing an override to `QueryIter::fold`, it was possible to port the iteration strategies of `Query::for_each` so that `Query::iter` and co could achieve the same gains. Not _every_ iterator method currently benefits from this, as they require overriding `QueryIter::try_fold`, but that is currently still a nightly-only optimisation. This same approach is within the Rust standard library.

This deduplicated code in a few areas, such as no longer requiring both `Query::for_each` and `Query::for_each_mut`, as one just needs to call `Query::iter` or `Query::iter_mut` instead. So code like:

```rust
fn some_system(mut q_transform: Query<&mut Transform, With<Npc>>) {
    q_transform.for_each_mut(|transform| {
        // Do something...
    });
}
```

Becomes:

```rust
fn some_system(mut q_transform: Query<&mut Transform, With<Npc>>) {
    q_transform.iter_mut().for_each(|transform| {
        // Do something...
    });
}
```

The assembly output was compared as well between what was on main branch versus the PR, with no tangible differences being seen between the old `Query::for_each` and the new `QueryIter::for_each()` output, validating the approach and ensuring the internal iteration optimizations were being applied.

As a plus, the same internal iteration optimizations in `Query::par_for_each` now reuse code from `for_each`, deduplicating code there as well and enabling users to make use of `par_iter().for_each()`. As a whole, this means there's no longer any need for `Query::for_each`, `Query::for_each_mut`, `Query::_par_for_each`, `Query::par_for_each_mut` so these methods have been deprecated for 0.13 and will be removed in 0.14.

## Reducing `TableRow` `as` Casting [#](https://bevy.org/news/bevy-0-13/#reducing-tablerow-as-casting)

authors: @bushrat011899

Not all improvements in our ECS internals were focused on performance. Some small changes were made to improve type safety and tidy up some of the codebase to have less `as` casting being done on various call sites for `TableRow`. The problem with `as` casting is that in some cases, the cast will fail by truncating the value silently, which could then cause havoc by accessing the wrong row and so forth. [#10811](https://github.com/bevyengine/bevy/pull/10811) by @bushrat011899 was put forward to clean up the API around `TableRow`, providing convenience methods backed by `assert`s to ensure the casting operations could never fail, or if they did, they'd panic correctly.

Naturally, _adding_ asserts in potentially hot codepaths were cause for some concern, necessitating considerable benchmarking efforts to confirm there were regressions and to what level. With careful placing of the new `assert`s, the detected regression for these cases was in the region of 0.1%, well within noise. But the benefit was a less error-prone API and more robust code. With a complex unsafe codebase like `bevy_ecs`, every little bit helps.

## Events Live Longer [#](https://bevy.org/news/bevy-0-13/#events-live-longer)

Events are a useful tool for passing data into systems and between systems.

Internally, Bevy events are double-buffered, so a given event will be silently dropped once the buffers have swapped twice. The `Events<T>` resource is set up this way so events are dropped after a predictable amount of time, preventing their queues from growing forever and causing a memory leak.

Before 0.12.1, event queues were swapped every update (i.e. every frame). That was an issue for games with logic in `FixedUpdate` since it meant events would normally disappear before systems in the next `FixedUpdate` could read them.

Bevy 0.12.1 changed the swap cadence to "every update that runs `FixedUpdate` one or more times" (only if the `TimePlugin` is installed). This change did resolve the original problem, but it then caused problems in the other direction. Users were surprised to learn some of their systems with `run_if` conditions would iterate much older events than expected. (In hindsight, we should have considered it a breaking change and postponed it until this release.) The change also introduced a bug (fixed in this release) where only one type of event was being dropped.

One proposed future solution to this lingering but unintended coupling between `Update` and `FixedUpdate` is to use event timestamps to change the default range of events visible by `EventReader<T>`. That way systems in `Update` would skip any events older than a frame while systems in `FixedUpdate` could still see them.

For now, the `<=0.12.0` behavior can be recovered by simply removing the `EventUpdateSignal` resource.

```rust
fn main() {
    let mut app = App::new()
        .add_plugins(DefaultPlugins);
    
    /* ... */

    // If this resource is absent, events are dropped the same way as <=0.12.0.
    app.world.remove_resource::<EventUpdateSignal>();
    
    /* ... */

    app.run();
}
```

## What's Next? [#](https://bevy.org/news/bevy-0-13/#what-s-next)

We have plenty of work in progress! Some of this will likely land in **Bevy 0.14**.

Check out the [**Bevy 0.14 Milestone**](https://github.com/bevyengine/bevy/milestone/20) for an up-to-date list of current work that contributors are focusing on for **Bevy 0.14**.

### More Editor Experimentation [#](https://bevy.org/news/bevy-0-13/#more-editor-experimentation)

Led by the brilliant JMS55, we've opened up a free-form [playground](https://github.com/bevyengine/bevy_editor_prototypes) to define and answer [key questions](https://github.com/bevyengine/bevy_editor_prototypes/discussions/1) about the design of the `bevy_editor`: not through discussion, but through concrete prototyping. Should we use an in-process editor (less robust to game crashes) or an external one (more complex)? Should we ship an editor binary (great for non-programmers) or embed it in the game itself (very hackable)? Let's find out by doing!

There are some incredible mockups, functional prototypes and third-party editor-adjacent projects out there. Some highlights:

[![A mockup of the UI for a graphical Bevy editor](https://bevy.org/news/bevy-0-13/editor_mockup.png)](https://bevy.org/news/bevy-0-13/editor_mockup.png) (1) bevy_editor_mockup

[![A node-based animation graph editor built with `bevy_egui`](https://bevy.org/news/bevy-0-13/locomotion_graph.png)](https://bevy.org/news/bevy-0-13/locomotion_graph.png) (2) bevy_animation_graph

[![A screenshot from `space_editor`, showcasing a functional scene editor with gizmos](https://bevy.org/news/bevy-0-13/space_editor.png)](https://bevy.org/news/bevy-0-13/space_editor.png) (3) space_editor

[![A screenshot from Blender, with Blender UI modifying Bevy component values](https://bevy.org/news/bevy-0-13/bevy_components.jpg)](https://bevy.org/news/bevy-0-13/bevy_components.jpg) (4) bevy_components

[![A recording of a web-based editor resulting in changes to Bevy entities in real time](https://bevy.org/news/bevy-0-13/makeshift_web.jpg)](https://bevy.org/news/bevy-0-13/makeshift_web.jpg) (5) bevy_remote

1. A Bevy-branded editor UI mockup by `@!!&Amy` on Discord, imagining what the UX for an ECS-based editor [could look like](https://amytimed.github.io/bevy_editor_mockup/editor/)
2. [`bevy_animation_graph`](https://crates.io/crates/bevy_animation_graph): a fully-functional asset-driven animation graph crate with its own node-based editor for Bevy
3. [`space_editor`](https://github.com/rewin123/space_editor): a polished Bevy-native third-party scene editor that you can use today!
4. [`Blender_bevy_components_workflow`](https://github.com/kaosat-dev/Blender_bevy_components_workflow): an impressively functional ecosystem of tools that lets you use Blender as a seamless level and scene editor for your games today.
5. `@coreh`'s experiment on a [reflection-powered remote protocol](https://github.com/coreh/bevy/pull/1), coupled with an interactive web-based editor, allows devs to inspect and control their Bevy games from other processes, languages and even devices! [Try it out live](https://makeshift-bevy-web-editor.vercel.app/)!

It's really exciting to see this progress, and we're keen to channel that energy and experience into official first-party efforts.

### `bevy_dev_tools` [#](https://bevy.org/news/bevy-0-13/#bevy-dev-tools)

The secret to smooth game development is great tooling. It's time to give Bevy developers the tools they need to inspect, debug and profile their games as part of the first-party experience. From FPS meters to system stepping to a first-party equivalent of the fantastic [`bevy-inspector-egui`](https://crates.io/crates/bevy-inspector-egui): giving these a home in Bevy itself helps us polish them, points new users in the right direction, and allows us to use them in the `bevy_editor` itself.

### A New Scene Format [#](https://bevy.org/news/bevy-0-13/#a-new-scene-format)

[Scenes](https://github.com/bevyengine/bevy/tree/latest/examples/scene) are Bevy's general-purpose answer to serializing ECS data to disk: tracking entities, components, and resources for both saving games and loading premade levels. However, the existing .ron-based scene format is hard to hand-author, overly verbose, and brittle; changes to your code (or that of your dependencies!) rapidly invalidate saved scenes. Cart has been cooking up a [revised scene format](https://github.com/bevyengine/bevy/discussions/9538) with tight IDE and code integration that tackles these problems and makes authoring content (including UI!) in Bevy a joy. Whether you're writing code, writing scene files, or generating it from a GUI.

### `bevy_ui` Improvements [#](https://bevy.org/news/bevy-0-13/#bevy-ui-improvements)

`bevy_ui` has its fair share of problems and limitations, [both mundane and architectural](https://www.leafwing-studios.com/blog/ecs-gui-framework/); however, there are tangible things we can and are doing to improve this: an improved scene format offers an end to the boilerplate when defining layouts, [rounded](https://github.com/bevyengine/bevy/pull/8973) [corners](https://github.com/bevyengine/bevy/pull/11813) just need a little love from reviewers, and the powerful and beloved object picking from [`bevy_mod_picking`] is slated to be upstreamed for both UI and gameplay alike. A spectacular array of [third-party UI solutions](https://bevy.org/assets/#ui) exists today, and learning from those and committing to a core architecture for UI logic and reactivity is a top priority.

### Meshlet Rendering [#](https://bevy.org/news/bevy-0-13/#meshlet-rendering)

Split meshes into clusters of triangles called meshlets, which bring many efficiency gains. During the 0.13 development cycle, we made a [lot of progress on this feature](https://github.com/bevyengine/bevy/pull/10164). We implemented a GPU-driven meshlet renderer that can scale to much more triangle-dense scenes, with a much lower CPU load. Memory usage, however, is very high, and we haven't implemented LODs or compression yet. Instead of releasing it half-baked, we're going to continue to iterate, and are very excited to (hopefully) bring you this feature in a future release.

![The Stanford dragon mesh rendered as meshlet clusters](https://bevy.org/news/bevy-0-13/meshlet_preview.png)

### The Steady March Towards Relations [#](https://bevy.org/news/bevy-0-13/#the-steady-march-towards-relations)

[Entity-entity relations](https://github.com/bevyengine/bevy/issues/3742), the ability to track and manage connections between entities directly in the ECS, has been one of the most requested ECS features for years now. Following the [trail blazed by `flecs`](https://ajmmertens.medium.com/building-games-in-ecs-with-entity-relationships-657275ba2c6c), the mad scientists over in `#ecs-dev` are steadily [reshaping our internals](https://github.com/orgs/bevyengine/projects/15), [experimenting with external implementations](https://crates.io/crates/aery), and shipping the general purpose building blocks (like dynamic queries or [lifecycle hooks](https://github.com/bevyengine/bevy/pull/10756)) needed to build a fast, robust and ergonomic solution.