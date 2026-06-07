#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <process.h>
#include <windows.h>

int main(int argc, char** argv) {
    char target_path[4096];
    
    DWORD len = GetModuleFileNameA(NULL, target_path, sizeof(target_path) - 1);
    if (len == 0) {
        perror("GetModuleFileNameA failed");
        return 1;
    }
    target_path[len] = '\0';
    
    char* last_slash = strrchr(target_path, '/');
    char* last_backslash = strrchr(target_path, '\\');
    char* separator = last_slash > last_backslash ? last_slash : last_backslash;
    
    if (separator) {
        *(separator + 1) = '\0';
        strncat(target_path, "clang-22.exe", sizeof(target_path) - strlen(target_path) - 1);
    } else {
        strcpy(target_path, "clang-22.exe");
    }

    char** new_argv = malloc((argc + 3) * sizeof(char*));
    int new_argc = 0;
    
    new_argv[new_argc++] = target_path;
    new_argv[new_argc++] = "-c";
    
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "--64") == 0) {
            new_argv[new_argc++] = "-m64";
        } else if (strcmp(argv[i], "--32") == 0) {
            new_argv[new_argc++] = "-m32";
        } else {
            new_argv[new_argc++] = argv[i];
        }
    }
    new_argv[new_argc] = NULL;

    intptr_t status = _spawnv(_P_WAIT, target_path, (const char* const*)new_argv);
    if (status == -1) {
        perror("spawnv failed");
        return 1;
    }
    return (int)status;
}
