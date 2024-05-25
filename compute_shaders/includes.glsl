
//Specialization constants
layout(constant_id = 0) const int canvas_size_x = 1;
layout(constant_id = 1) const int canvas_size_y = 1;
layout(constant_id = 2) const uint empty_matter = 1;
layout(local_size_x_id = 3, local_size_y_id = 4, local_size_z = 1) in;

//falling spec constants
layout(push_constant) uniform PushConstants {
    uint sim_step;
    uint move_step;
} push_constants;

//Buffers
layout(set = 0, binding = 0) restrict buffer MatterInBuffer { uint matter_in[]; };
layout(set = 0, binding = 1) restrict writeonly buffer MatterOutBuffer { uint matter_out[]; };
layout(set = 0, binding = 2, rgba8) restrict uniform writeonly image2D canvas_img;

#include "matter.glsl"

//general utility functions

ivec2 get_current_sim_pos() {
    return ivec2(gl_GlobalInvocationID.xy);
}

int get_index(ivec2 pos) {
    return pos.y * canvas_size_x + pos.x;
}

bool is_inside_sim_canvas(ivec2 pos) {
    return pos.x >= 0 && pos.x < canvas_size_x &&
    pos.y >= 0 && pos.y < canvas_size_y;
}

Matter read_matter(ivec2 pos) {
    return new_matter(matter_in[get_index(pos)]);
}

uint matter_to_uint(Matter matter) {
    return ((matter.color << uint(8)) | matter.matter);
}

void write_matter(ivec2 pos, Matter matter) {
    matter_out[get_index(pos)] = matter_to_uint(matter);
}

void write_image_color(ivec2 pos, vec4 color) {
    imageStore(canvas_img, pos, color);
}

// utility functions falling sand

#include "dirs.glsl"

bool is_at_border_top(ivec2 pos) {
    return pos.y == canvas_size_y - 1;
}

bool is_at_border_bottom(ivec2 pos) {
    return pos.y == 0;
}

bool is_at_border_right(ivec2 pos) {
    return pos.x == canvas_size_x - 1;
}

bool is_at_border_left(ivec2 pos) {
    return pos.x == 0;
}

ivec2 get_pos_at_dir(ivec2 pos, int dir) {
    return pos + OFFSETS[dir];
}

// | 0 1 2 |
// | 7 x 3 |
// | 6 5 4 |
Matter get_neighbor(ivec2 pos, int dir) {
    ivec2 neighbor_pos = get_pos_at_dir(pos, dir);
    if (is_inside_sim_canvas(neighbor_pos)) {
        return read_matter(neighbor_pos);
    } else {
        return new_matter(empty_matter);
    }
}

//shortcut for water (not added)
bool is_fluid(Matter matter){
    return matter.matter == 3;
}

bool is_empty(Matter matter) {
    return matter.matter == 0;
}

//anything that is above a certain number has gravity
bool has_gravity(Matter m) {
    return m.matter > 1;
}

bool falls_on_empty(Matter from, Matter to) {
    return has_gravity(from) && is_empty(to);
}

//diagonal movement

bool slides_on_empty(Matter from_diagonal, Matter to_diagonal, Matter from_down) {
    return has_gravity(from_diagonal) && !is_empty(from_down) && is_empty(to_diagonal);
}

bool moves_on_empty(Matter from , Matter to , Matter down, Matter down_side){
    return is_fluid(from) && is_empty(to) && !is_empty(down) && !is_empty(down_side);
}
