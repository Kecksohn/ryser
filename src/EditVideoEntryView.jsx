import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export const EditVideoEntryView = ({disable_view, update_element_in_library, video_entry}) => {

    const [title_input, set_title_input] = useState(video_entry.title);
    const [year_input, set_year_input] = useState(video_entry.year);
    const [director_input, set_director_input] = useState(video_entry.director);
    const [countries_input, set_countries_input] = useState(video_entry.countries);
    const [new_image_url_or_path, set_new_poster_url_or_path] = useState("");

    function update_element() {
        if (video_entry.title === title_input
            && video_entry.year === year_input
            && video_entry.director === director_input
            && video_entry.countries === countries_input
        ) {
            console.log("No changes found.");
            return;
        }
        video_entry.title = title_input;
        video_entry.year = year_input;
        video_entry.director = director_input;
        video_entry.countries = countries_input;
        update_element_in_library(video_entry);
    }

    return(
        <div>
            Filepath: {video_entry.filepath}
            <br/>Title: <input id="titleinput" type="text"
                className="w-full px-3 py-2 border rounded-md border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
                value={title_input} onChange={(e) => set_title_input(e.target.value)}/>
            <div>Year: {video_entry.year}</div>
            <div>Director: {video_entry.director}</div>
            <div>Countries: {video_entry.countries}</div>
            <div>Watched: {video_entry.watched}</div>
            <div>Poster Path: {video_entry.poster_path}</div>

            <br/>
            <div style={{cursor: "pointer"}} onClick={() => {update_element(); disable_view();}}>Update</div>
            <div style={{cursor: "pointer"}} onClick={() => {disable_view();}}>Back</div>
        </div>
    )

}