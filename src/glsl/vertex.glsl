#version 300 es

in vec2 a_pos;
out vec2 v_uv;

void main() {
    // [-1, 1] の範囲を [0, 1] に変換する
    v_uv = (a_pos + 1.0) * 0.5;
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
