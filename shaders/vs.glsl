// those are our vertex attributes
in vec3 position;
in vec3 color;

// this is the output of the vertex shader (we could have had several ones)
out vec3 v_color;

uniform mat4 mvp;

void main() {
    // simply forward the color
    v_color = color;

    // mandatory; tell the GPU to use the position vertex attribute to put the vertex in space
    gl_Position = mvp * vec4(position, 1.0);
}

