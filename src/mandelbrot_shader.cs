#version 450

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

void main() {
    vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
    vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(5.0, 0.0);

    vec2 z = vec2(0.0, 0.0);
    float i;
    for (i = 0.0; i < 1.0; i += 0.004) {
        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            z.y * z.x + z.x * z.y + c.y
        );

        if (length(z) > 4.0) {
            break;
        }
    }

    vec4 to_write;

    if (i <= 0.3){
        to_write = vec4(0.0, 0.0, i, 1.0);
    }else if(i <= 0.6){
        to_write = vec4(0.0, i, 1.0, 1.0 );
    }else if(i + 0.004 >= 1.0){
        to_write = vec4(0.0, 0.0, 0.0, 1.0 );
    }else{
        to_write = vec4(i, 1.0, 1.0, 1.0 );
    }

    imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
}