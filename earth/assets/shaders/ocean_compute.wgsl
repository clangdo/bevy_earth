// References (more specific references listed below):
// - https://gpuweb.github.io/gpuweb/wgsl/

struct OceanParameters {
    wind: vec2<f32>,
    time: f32,
}

struct WaveSample {
    displacement: vec4<f32>,
    normal: vec4<f32>,
}

const NUM_WAVES = 25;

const JITTER_FACTOR = 0.25;
const PI = 3.14159265358979323846;

// The patch size in meters
const PATCH_SIZE = vec2<f32>(25.0, 25.0);

@group(0) @binding(0)
var seed: texture_2d<f32>;

@group(0) @binding(1)
var displacement_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var normal_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(3)
var<uniform> parameters: OceanParameters;

/* References for waves:
 * - https://en.wikipedia.org/wiki/Trochoidal_wave
 * - Tessendorf, Jerry. (2001). Simulating Ocean Water. SIG-GRAPH'99 Course Note.
 * - Horvath, Christopher J. (2015). Empirical directional wave
 *    spectra for computer graphics. Proceedings of the 2015 Symposium on
 *    Digital Production, 29–39. https://doi.org/10.1145/2791261.2791267 */

// The following function impelements Donelan-Banner Directional
// Spreading, detailed in section 6.0.9 of Horvath's paper.
//
// For now, this does truncate at 0.56, unlike the implementation
// notes in Horvath's paper denote.
fn directional_spread(angular_frequency_part: f32, wind_relative_wave_dir: f32) -> f32 {
    var beta = 0.0;
    if angular_frequency_part >= 1.6 {
        let exponent = -0.567 * log(pow(angular_frequency_part, 2.0));
        let epsilon = -0.4 + 0.8393 * exp(exponent);
        beta = pow(10.0, epsilon);
    } else if angular_frequency_part >= 0.95 {
        beta = 2.28 * pow(angular_frequency_part, -1.3);
    } else if angular_frequency_part >= 0.56 {
        beta = 2.61 * pow(angular_frequency_part, 1.3);
    }

    return beta / 2.0 * pow(cosh(beta * wind_relative_wave_dir), -2.0)/tanh(beta * PI);
}

fn jonswap_spectrum(angular_freq: f32) -> f32 {
    // TODO!
    return 0.0;
}

fn sample_wave(in_pos: vec2<f32>, random: vec2<f32>) -> WaveSample {
    let jitter = random * JITTER_FACTOR;
    let wind = parameters.wind * jitter + parameters.wind;
    let amplitude = 0.5;
    let angular_frequency = 1.0;
    let frequency = (1.0 + length(jitter)) * length(wind / wind.x) * PI;
    let temporal_phase = parameters.time * angular_frequency;
    let locative_phase = frequency * dot(in_pos, wind);

    // This is equivalent to \frac{\vec{k}}{k}} in Tessendorf's notes.
    let wind_dir = normalize(wind);
    // This is equivalent to \vec{k} \dot \vec{x} in Tessendorf's notes.
    let phase = locative_phase - temporal_phase;

    // The equation from Tessendorf's notes
    let lateral_offset = - wind_dir * amplitude * sin(phase);
    let vertical_offset = amplitude * cos(phase);
    var displacement = vec4<f32>(lateral_offset, vertical_offset, 1.0);

    displacement += vec4<f32>(vec3<f32>(0.5), 1.0);

    // X tangent vector
    let dsdx = vec3<f32>(
        1.0 - wind_dir.x * amplitude * cos(phase) * wind.x,
        0.0,
        -amplitude * sin(phase) * wind.x,
    );

    // Y tangent vector
    let dsdy = vec3<f32>(
        0.0,
        1.0 - wind_dir.y * amplitude * cos(phase) * wind.y,
        -amplitude * sin(phase) * wind.y,
    );

    // Cross the tangent vectors to get the normal
    let normal = vec4<f32>(cross(dsdx, dsdy) / 0.5 + vec3<f32>(0.5), 1.0);

    return WaveSample(displacement, normal);
}

fn calculate_texel(coord: vec2<i32>) {
    let in_pos = PATCH_SIZE * vec2<f32>(coord) / vec2<f32>(textureDimensions(normal_texture));

    let seed_width = textureDimensions(seed).x;
    var displacement = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var normal = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    for(var i = 0; i < NUM_WAVES; i++) {
        let iteration_coordinate = vec2<i32>(i / seed_width, i % seed_width);
        let random = textureLoad(seed, iteration_coordinate, 0);

        let sample = sample_wave(in_pos, random.xy);
        displacement += sample.displacement;
        normal += sample.normal;
    }

    displacement /= f32(NUM_WAVES);
    normal /= f32(NUM_WAVES);

    textureStore(displacement_texture, coord, displacement);
    textureStore(normal_texture, coord, normal);
}

@compute @workgroup_size(16, 16)
fn compute(@builtin(local_invocation_id) coords: vec3<u32>, @builtin(local_invocation_index) idx: u32) {
    let coord = 16 * vec2<i32>(coords.xy);

    for (var i = 0; i < 16; i++) {
        for (var j = 0; j < 16; j++) {
            calculate_texel(coord + vec2<i32>(i, j));
        }
    }   
}

