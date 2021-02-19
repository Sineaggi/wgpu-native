#ifndef WGPU_H
#define WGPU_H
#include "wgpu.h"
#endif

#include <stdio.h>
#include <stdlib.h>

WGPUShaderModuleDescriptor load_shader(const char *name) {
    FILE *file = fopen(name, "rb");
    if (!file) {
        printf("Unable to open %s\n", name);
        exit(1);
    }
    fseek(file, 0, SEEK_END);
    long length = ftell(file);
    char *source = malloc(length + 1);
    fseek(file, 0, SEEK_SET);
    fread(source, 1, length, file);
    fclose(file);
    source[length] = '\0';

    WGPUShaderModuleWGSLDescriptor *wgslDescriptor = malloc(sizeof(WGPUShaderModuleWGSLDescriptor));
    wgslDescriptor->chain = (WGPUChainedStruct) {
        .next = NULL,
        .s_type = WGPUSType_ShaderModuleWGSLDescriptor
    };
    wgslDescriptor->source = source;
    return (WGPUShaderModuleDescriptor) {
        .next_in_chain = (const WGPUChainedStruct *) wgslDescriptor,
        .label = NULL,
        .flags = WGPUShaderFlags_VALIDATION,
    };
}

void free_shader(WGPUShaderModuleDescriptor *shaderModuleDescriptor) {
    WGPUShaderModuleWGSLDescriptor *wgslDescriptor = shaderModuleDescriptor->next_in_chain;
    free(wgslDescriptor->source);
    shaderModuleDescriptor->next_in_chain = NULL;
    free(wgslDescriptor);
    shaderModuleDescriptor->next_in_chain = NULL;
}
