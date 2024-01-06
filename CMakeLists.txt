cmake_minimum_required (VERSION 3.12)

# Enable Hot Reload for MSVC compilers if supported.
if (POLICY CMP0141)
  cmake_policy(SET CMP0141 NEW)
  set(CMAKE_MSVC_DEBUG_INFORMATION_FORMAT "$<IF:$<AND:$<C_COMPILER_ID:MSVC>,$<CXX_COMPILER_ID:MSVC>>,$<$<CONFIG:Debug,RelWithDebInfo>:EditAndContinue>,$<$<CONFIG:Debug,RelWithDebInfo>:ProgramDatabase>>")
endif()

project ("ryser")
set(CMAKE_INCLUDE_CURRENT_DIR ON)

add_executable (${PROJECT_NAME} "src/main.cpp" "src/videoplayer.cpp" "src/manageconfig.cpp")

set_property(TARGET ${PROJECT_NAME} PROPERTY CXX_STANDARD 20)


# Boost
find_package(Boost REQUIRED)
target_include_directories(${PROJECT_NAME} PRIVATE ${Boost_INCLUDE_DIRS})
target_link_libraries(${PROJECT_NAME} PRIVATE ${Boost_LIBRARIES})




# Copy config.json to build directory
add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
                   COMMAND ${CMAKE_COMMAND} -E copy_if_different
                   ${CMAKE_SOURCE_DIR}/config.json
                   $<TARGET_FILE_DIR:${PROJECT_NAME}>)