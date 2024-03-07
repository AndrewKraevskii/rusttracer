struct VertexOutput {
    @location(0) position: vec4<f32>,
    @builtin(position) screen_coord: vec4<f32>,
};


struct Sphere {
    center: vec3f,
    radius: f32,
};

struct Ray {
    start: vec3f,
    direction: vec3f,
};

struct IntersectionPoint {
    distance: f32,
    point: vec3f,
    normal: vec3f,
};
struct PerspectiveCamera {
    position: vec3f,
    right: vec3f,
    up: vec3f,
    height: f32,
    width: f32,
    focal_distance: f32,
}

const camera = PerspectiveCamera(
    vec3f(0, 0, 20),
    vec3f(25, 0, 0),
    vec3f(0, 25, 0),
    320,
    320,
    50,
);

fn top_left() -> vec3f {
    return camera.position - normalize(camera.right) * camera.width +  normalize(camera.up) * camera.height; 
}
fn top_right() -> vec3f {
    return camera.position + normalize(camera.right) * camera.width +  normalize(camera.up) * camera.height; 
}
fn bottom_left() -> vec3f {
    return camera.position - normalize(camera.right) * camera.width -  normalize(camera.up) * camera.height; 
}
fn bottom_right() -> vec3f {
    return camera.position + normalize(camera.right) * camera.width -  normalize(camera.up) * camera.height; 
}
fn front() -> vec3f {
    return cross(camera.right, camera.up);
}
fn get_ray(x: f32, y: f32) -> Ray {
    let start = mix(
        mix(top_left(), top_right(), x),
        mix(bottom_left(), bottom_right(), x),
        y,
    );
    return Ray(
        start,
        start - (camera.position -
                normalize(front()) *
                camera.focal_distance)
    );
}


@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var vertices = array<vec2<f32>, 6>(
        vec2<f32>(-1, -1),
        vec2<f32>(-1, 1),
        vec2<f32>(1, 1),
        vec2<f32>(1, 1),
        vec2<f32>(1, -1),
        vec2<f32>(-1, -1),
    );
    
    var result: VertexOutput;
    result.position = vec4<f32>(vertices[in_vertex_index], 0.0, 1.0);
    result.screen_coord = result.position ;
    return result;
}

fn sq(v: f32) -> f32 {
    return v * v;
}

fn intersectRaySphere(ray: Ray, sphere: Sphere) -> array<IntersectionPoint, 2> {
    let discriminant: f32 = sq(dot(ray.direction, ray.start - sphere.center)) +
        dot(ray.direction, ray.direction) * (sq(sphere.radius) - dot(ray.start, ray.start) +
        2 * dot(ray.start, sphere.center) - dot(sphere.center, sphere.center));
    if (discriminant < 0) {
        discard;
    }
    let res: f32 = dot(ray.direction, (sphere.center - ray.start)) / dot(ray.direction, ray.direction);
    let distance = array<f32, 2>(
        res - sqrt(discriminant) / dot(ray.direction, ray.direction),
        res + sqrt(discriminant) / dot(ray.direction, ray.direction),
    );

    let normals = array<vec3f, 2>(
        normalize(ray.start + ray.direction * distance[0] - sphere.center),
        normalize(ray.start + ray.direction * distance[1] - sphere.center),
    );

    return array<IntersectionPoint, 2>(
            IntersectionPoint(
               distance[0],
               normals[0],
             ray.start + ray.direction * distance[0],
            ),
            IntersectionPoint(
               distance[1],
               normals[1],
             ray.start + ray.direction * distance[1],
            )
        );
}

struct UniformState {
    time: f32,
    apsect: f32, // w/h
}

@group(0)
@binding(0)
var<uniform> state: UniformState;

const sphere = Sphere(
    vec3f(0, 0, 0),
    20
);

@fragment
fn fs_main(
    vertex: VertexOutput,
) -> @location(0) vec4<f32> {
    let pos_x = (vertex.position.x + 1) / 2;
    let pos_y = (vertex.position.y + 1) / 2;

    let ray = get_ray(pos_x, pos_y + sin(state.time) / 2);
    let intersect = intersectRaySphere(ray, sphere);

    let light = normalize(vec3f(sin(state.time), cos(state.time), 1));

    
    return vec4<f32>(max(vec2f(dot(-light, normalize(intersect[0].normal))), vec2f(0.2)), sin(state.time), 1.0);
}

