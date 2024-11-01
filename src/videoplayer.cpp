#include "pch.hh"
#include "videoplayer.hh"

#include <iostream>
#include <windows.h>
#include <boost/property_tree/json_parser.hpp>
#include <boost/property_tree/ptree_fwd.hpp>


// TODO: Windows only
std::wstring utf8_to_utf16(const std::string& utf8)
{
    std::wstring utf16;
    int size = MultiByteToWideChar(CP_UTF8, 0, utf8.data(), utf8.size(), NULL, 0);
    utf16.resize(size);
    if (size > 0)
        MultiByteToWideChar(CP_UTF8, 0, utf8.data(), utf8.size(), &utf16[0], size);
    return utf16;
}


void launch_videoplayer(std::string const& video_filepath, bool wait_for_close)
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

    //const LPCSTR executable_path = executable_path_str.c_str();
    //const LPSTR args = const_cast<char*>(args_str.c_str());

    std::wstring executable_path_wstr = utf8_to_utf16(executable_path_str);
    LPCWSTR executable_path = executable_path_wstr.c_str();

    // Convert std::string to LPWSTR
    std::wstring args_wstr = utf8_to_utf16(args_str);
    LPWSTR args = const_cast<LPWSTR>(args_wstr.c_str());

    STARTUPINFOW si;
    PROCESS_INFORMATION pi;
    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);
    ZeroMemory(&pi, sizeof(pi));
    // Start the child process.
    if (!CreateProcessW(
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
        printf("CreateProcess failed (%d).\n", (int)GetLastError());
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
