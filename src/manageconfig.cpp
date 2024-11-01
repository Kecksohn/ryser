#include "pch.hh"
#include "manageconfig.hh"

#include <iostream>
#include <boost/property_tree/json_parser.hpp>


bool load_json(boost::property_tree::ptree& pt)
{
	try
	{
		boost::property_tree::json_parser::read_json("config.json", pt);
		return true;
	}
	catch (const boost::property_tree::json_parser_error& e)
	{
		std::cerr << "Failed to read " << "config.json" << ": " << e.what() << '\n';
		return false;
	}
}

bool save_json(boost::property_tree::ptree& pt)
{
	try
	{
		boost::property_tree::json_parser::write_json("config.json", pt);
		return true;
	}
	catch (const boost::property_tree::json_parser_error& e)
	{
		std::cerr << "Failed to write " << "config.json" << ": " << e.what() << '\n';
		return false;
	}
}



void change_videoplayer(std::string const& video_filepath, std::string const& args)
{
	boost::property_tree::ptree pt;
	if(!load_json(pt)) return;
	pt.put("VideoPlayer.executable_path", video_filepath);
	pt.put("VideoPlayer.args", args);
	save_json(pt);
}



void add_library(std::string const& library_name, std::string const& library_path, bool const& recursive)
{
	boost::property_tree::ptree pt;
	if (!load_json(pt)) return;
	boost::property_tree::ptree library;
	library.put(library_path + ".recursive", recursive);
	pt.get_child("Libraries").push_back(std::make_pair(library_name, library));
	save_json(pt);
}

void remove_library(std::string const& library_name)
{
	boost::property_tree::ptree pt;
	if (!load_json(pt)) return;
	pt.get_child("Libraries").erase(library_name);
	save_json(pt);
}

void add_path_to_library(std::string const& library_name, std::string const& new_path, bool const& recursive)
{
	boost::property_tree::ptree pt;
	if (!load_json(pt)) return;
	pt.get_child("Libraries").get_child(library_name).put(new_path + ".recursive", recursive);
	save_json(pt);
}


