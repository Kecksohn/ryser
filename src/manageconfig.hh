#pragma once
#include <string>
#include <boost/property_tree/json_parser.hpp>

bool load_json(boost::property_tree::ptree& pt);

void change_videoplayer(std::string const& video_filepath, std::string const& args);

void add_library(std::string const& library_name, std::string const& library_path, bool const& recursive = false);
void remove_library(std::string const& library_name);
void add_path_to_library(std::string const& library_name, std::string const& new_path, bool const& recursive = false);