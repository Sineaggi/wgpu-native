#ifndef WGPU_H
#define WGPU_H
#include "wgpu.h"
#endif

#include <stdio.h>
#include <stdlib.h>

WGPUShaderModuleDescriptor read_file(const char *name) {
    FILE *file = fopen(name, "rb");
    if (!file) {
        printf("Unable to open %s\n", name);
        exit(1);
    }
    fseek(file, 0, SEEK_END);
    long length = ftell(file);
    char *bytes = malloc(length + 1);
    fseek(file, 0, SEEK_SET);
    fread(bytes, 1, length, file);
    fclose(file);
    bytes[length] = 0;

    WGPUShaderModuleWGSLDescriptor *wgslDescriptor = malloc(sizeof(WGPUShaderModuleWGSLDescriptor));
    wgslDescriptor->chain = (WGPUChainedStruct) {
        .next = NULL,
        .s_type = WGPUSType_ShaderModuleWGSLDescriptor
    };
    wgslDescriptor->source = bytes;
    return (WGPUShaderModuleDescriptor) {
        .next_in_chain = (const WGPUChainedStruct *) wgslDescriptor,
        .label = NULL,
        .flags = WGPUShaderFlags_VALIDATION,
    };
}
