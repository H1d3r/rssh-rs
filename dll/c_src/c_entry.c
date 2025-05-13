#include <Windows.h>
void c_entry(){
    CHAR msg[] = "Hello from FFI C code!";
    MessageBoxA(NULL, msg, msg, 0);
}