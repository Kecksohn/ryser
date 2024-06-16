cmake_minimum_required (VERSION 3.16)

# Enable Hot Reload for MSVC compilers if supported.
if (POLICY CMP0141)
  cmake_policy(SET CMP0141 NEW)
  set(CMAKE_MSVC_DEBUG_INFORMATION_FORMAT "$<IF:$<AND:$<C_COMPILER_ID:MSVC>,$<CXX_COMPILER_ID:MSVC>>,$<$<OR:$<CONFIG:Debug>,$<CONFIG:RelWithDebInfo>>:EditAndContinue>,$<$<OR:$<CONFIG:Debug>,$<CONFIG:RelWithDebInfo>>:ProgramDatabase>>")
endif()

project ("ryser")
set(PROJECT_ROOT ${CMAKE_CURRENT_SOURCE_DIR})

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
cmake_policy(SET CMP0100 NEW)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)

add_executable (
    ${PROJECT_NAME} 
    "src/main.cpp"
    "src/videoplayer.cpp"
    "src/manageconfig.cpp"

    "src/QtUI/MainWindow.cpp"
    "src/QtUI/HomeView.cpp"
    "src/QtUI/HomeView.hh"
    "src/QtUI/LibraryView.cpp"
    "src/QtUI/LibraryView.hh"
)

target_include_directories(${PROJECT_NAME} PRIVATE
        ${PROJECT_ROOT}/src
)


if(MSVC)
  set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /Zc:__cplusplus")
  add_compile_options(/permissive-)
endif()

# Boost
find_package(Boost REQUIRED)
target_include_directories(${PROJECT_NAME} PRIVATE ${Boost_INCLUDE_DIRS})
target_link_libraries(${PROJECT_NAME} PRIVATE ${Boost_LIBRARIES})

# QT
find_package(Qt6 COMPONENTS Core Gui Widgets REQUIRED)
target_link_libraries(${PROJECT_NAME} PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets)
#message("Qt path: ${Qt6Core_DIR}")


# Copy config.json to build directory
add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
                   COMMAND ${CMAKE_COMMAND} -E copy_if_different
                   ${CMAKE_SOURCE_DIR}/config.json
                   $<TARGET_FILE_DIR:${PROJECT_NAME}>)

# Copy /styles/ folder to build directory
add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
                   COMMAND ${CMAKE_COMMAND} -E copy_directory
                   ${CMAKE_SOURCE_DIR}/styles/
                   $<TARGET_FILE_DIR:${PROJECT_NAME}>/styles/)