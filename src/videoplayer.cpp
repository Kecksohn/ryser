#include "videoplayer.hh"

#include <iostream>
#include <windows.h>
#include <boost/property_tree/json_parser.hpp>
#include <boost/property_tree/ptree_fwd.hpp>

void launch_videoplayer(const char* video_filepath, bool wait_for_close)
{

    boost::property_tree::ptree pt;
    try
    {
        boost::property_tree::json_parser::read_json("config.json", pt);
    }
    catch (const boost::property_tree::json_parser_error& e)
    {
        std::cerr << "Failed to read config.json: " << e.what() << '\n';
        return;
    }

    const std::string executable_path_str = pt.get<std::string>("VideoPlayer.executable_path");
    const std::string args_str = std::string(" \"") + video_filepath + std::string("\" ") + pt.get<std::string>("VideoPlayer.args");

    const LPCSTR executable_path = executable_path_str.c_str();
    const LPSTR args = const_cast<char*>(args_str.c_str());

    STARTUPINFO si;
    PROCESS_INFORMATION pi;
    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);
    ZeroMemory(&pi, sizeof(pi));
    // Start the child process.
    if (!CreateProcess(
        executable_path, // Application path
        args, // Command line arguments
        NULL, // Process handle not inheritable
        NULL, // Thread handle not inheritable
        FALSE, // Set handle inheritance to FALSE
        0, // No creation flags
        NULL, // Use parent's environment block
        NULL, // Use parent's starting directory
        &si, // Pointer to STARTUPINFO structure
        &pi) // Pointer to PROCESS_INFORMATION structure
        )
    {
        printf("CreateProcess failed (%d).\n", GetLastError());
        return;
    }

    if (!wait_for_close)
        return;

    // Wait until child process exits.
    WaitForSingleObject(pi.hProcess, INFINITE);
    // Close process and thread handles.
    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
}
