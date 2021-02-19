#ifndef WGPU_H
#define WGPU_H
#include "wgpu.h"
#endif

WGPUShaderModuleDescriptor load_shader(const char *name);

void free_shader(WGPUShaderModuleDescriptor *shaderModuleDescriptor);

void read_buffer_map(
    WGPUBufferMapAsyncStatus status,
    uint8_t *userdata);
