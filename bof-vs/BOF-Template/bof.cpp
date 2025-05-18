#include <Windows.h>
#include "base\helpers.h"

/**
 * For the debug build we want:
 *   a) Include the mock-up layer
 *   b) Undefine DECLSPEC_IMPORT since the mocked Beacon API
 *      is linked against the the debug build.
 */
#ifdef _DEBUG
#undef DECLSPEC_IMPORT
#define DECLSPEC_IMPORT
#include "base\mock.h"
#endif

// Real pipe name is stomped in by cna
static const char pipename[] = "\\\\.\\pipe\\INPUT_PIPE_NAME_NO_CHANGE_PLS\0\0\0\0";


extern "C" {
#include "beacon.h"
#include "sleepmask.h"

    void WriteToNamedPipe(char* buffer, DWORD buffer_size) {

        DFR_LOCAL(KERNEL32, CreateFileA)
        DFR_LOCAL(KERNEL32, CloseHandle)
        DFR_LOCAL(KERNEL32, WriteFile)
        DFR_LOCAL(KERNEL32, GetLastError)

        HANDLE hPipe = CreateFileA(
            pipename,  // pipe name
            GENERIC_WRITE,                                // write access
            0,                                            // no sharing
            NULL,                                         // default security attributes
            OPEN_EXISTING,                                // opens existing pipe
            0,                                            // default attributes
            NULL);                                        // no template file

        if (hPipe == INVALID_HANDLE_VALUE) {
            BeaconPrintf(CALLBACK_ERROR, "Failed to connect to pipe. Error: %lu\n", GetLastError());
            return;
        }

        DWORD bytesWritten = 0;
        BOOL success = WriteFile(
            hPipe,            // handle to pipe
            buffer,           // buffer to write
            buffer_size,      // size of buffer
            &bytesWritten,    // number of bytes written
            NULL);            // not overlapped I/O

        if (!success || bytesWritten != buffer_size) {
            BeaconPrintf(CALLBACK_ERROR, "Failed to write to pipe. Error: %lu\n", GetLastError());
        }

        CloseHandle(hPipe);
    }

    void go(char* args, int len) {
        char* data;
        int dataLen;
        datap parser;

        // Get the contents of the named pipe
        BeaconDataParse(&parser, args, len);
        data = BeaconDataExtract(&parser, &dataLen);

        // Send the data to the named pipe
        WriteToNamedPipe(data, dataLen);

        BeaconPrintf(CALLBACK_OUTPUT, "Sent command: %s", data);

    }

}

// Define a main function for the bebug build
#if defined(_DEBUG) && !defined(_GTEST)

int main(int argc, char* argv[]) {
    // Run BOF's entrypoint
    // To pack arguments for the bof use e.g.: bof::runMocked<int, short, const char*>(go, 6502, 42, "foobar");
    bof::runMocked<>(go, "exit", 5);

    return 0;
}

// Define unit tests
#elif defined(_GTEST)
#include <gtest\gtest.h>

TEST(BofTest, Test1) {
    std::vector<bof::output::OutputEntry> got =
        bof::runMocked<>(go, "exit", 5);
}
#endif