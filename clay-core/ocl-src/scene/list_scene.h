#pragma once

#include <clay_core/random.h>


#define SCENE_ARGS_DEF \
    __global const int *object_buffer_int, \
    __global const float *object_buffer_float, \
    int object_size_int, \
    int object_size_float, \
    int objects_count, \
    \
    __global const int *target_buffer_int, \
    __global const float *target_buffer_float, \
    int target_size_int, \
    int target_size_float, \
    int targets_count

#define SCENE_ARGS \
    object_buffer_int, \
    object_buffer_float, \
    object_size_int, \
    object_size_float, \
    objects_count, \
    \
    target_buffer_int, \
    target_buffer_float, \
    target_size_int, \
    target_size_float, \
    targets_count


#define MAX_DEPTH 4

#define TARGET_THRESHOLD 0.1f


bool scene_trace(
    uint *seed,
    Ray ray,
    Ray *new_ray,
    float3 *color,
    SCENE_ARGS_DEF
) {
    int hit_idx = -1;
    float hit_enter = INFINITY;
    float hit_exit = 0.0f;
    float3 hit_norm;

    int i = 0;
    for (i = 0; i < objects_count; ++i) {
        float enter, exit;
        float3 norm;

        if (ray.origin == i) {
            continue;
        }
        
        __global const int *ibuf = object_buffer_int + object_size_int*i;
        __global const float *fbuf = object_buffer_float + object_size_float*i;
        if (__object_hit(seed, ray, ibuf, fbuf, &enter, &exit, &norm)) {
            if (enter < hit_enter) {
                hit_enter = enter;
                hit_exit = exit;
                hit_norm = norm;
                hit_idx = i;
            }
        }
    }
    
    if (hit_idx >= 0) {
        if (
            (ray.type == RAY_DIFFUSE && ray.target == hit_idx) ||
            (ray.type == RAY_TARGET && ray.target != hit_idx)
        ) {
            return 0;
        }

        float3 hit_pos = ray.start + ray.dir*hit_enter;

        __global const int *ibuf = object_buffer_int + object_size_int*hit_idx;
        __global const float *fbuf = object_buffer_float + object_size_float*hit_idx;
        bool bounce = __object_emit(
            seed, ray, hit_pos, hit_norm,
            ibuf, fbuf, new_ray, color
        );
        if (bounce) {
            new_ray->origin = hit_idx;

            // Attraction
            /*
            if (new_rays[0].type == RAY_DIFFUSE) {
                int attract_idx = (int)(random_uniform(seed)*targets_count);
                __global const int *aibuf = targets_int + target_size_int*attract_idx;
                __global const float *afbuf = targets_float + target_size_float*attract_idx;

                int target = aibuf[0];
                Ray attract_ray = ray_new();
                float weight = 0.0f;
                int ret = __attract(
                    seed, new_rays[0], hit_norm, TARGET_THRESHOLD,
                    aibuf, afbuf, &attract_ray, &weight
                );
                if (ret >= 0) {
                    new_rays[0].target = target;
                    new_rays[0].color *= (1.0f - weight);
                    if (ret == 1) {
                        attract_ray.origin = hit_idx;
                        attract_ray.target = target;
                        new_rays[1] = attract_ray;
                        rays_count += 1;
                    }
                }
            }
            */

            return true;
        } else {
            return false;
        }
    } else {
        float z = 0.5f*(ray.dir.z + 1.0f);
        *color += ray.color*z*(float3)(0.2, 0.2, 0.4);
        return false;
    }
}

float3 __scene_trace(
    uint *seed,
    Ray ray,
    SCENE_ARGS_DEF
) {
    float3 color = (float3)(0.0f);
    int i = 0;
    Ray current_ray = ray;
    for (i = 0; i < MAX_DEPTH; ++i) {
        Ray next_ray = ray_new();
        bool bounce = scene_trace(seed, current_ray, &next_ray, &color, SCENE_ARGS);
        if (!bounce) {
            break;
        }
        current_ray = next_ray;
    }
    return color;
}
