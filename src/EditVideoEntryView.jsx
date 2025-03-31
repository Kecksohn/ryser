import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

import "./TMDBResults.css";

export const EditVideoEntryView = ({disable_view, update_element_in_library, video_entry}) => {

    const [title_input, set_title_input] = useState(video_entry.title ? video_entry.title : "");
    const [year_input, set_year_input] = useState(video_entry.year ? video_entry.year : "");
    const [director_input, set_director_input] = useState(video_entry.director ? video_entry.director : "");
    const [countries_input, set_countries_input] = useState(video_entry.countries ? video_entry.countries : [""]);
    const [new_image_url_or_path, set_new_poster_url_or_path] = useState(video_entry.poster_path);
    const [was_changed, set_was_changed] = useState(false);

    function commit_changes() {
        if (video_entry.title === title_input
            && video_entry.year === year_input
            && video_entry.director === director_input
            && video_entry.countries === countries_input
        ) {
            console.log("No changes found.");
            return;
        }
        video_entry.title = title_input;
        video_entry.year = year_input !== "" ? parseInt(year_input) : null;
        video_entry.director = director_input;
        video_entry.countries = countries_input;
        video_entry.poster_path = new_image_url_or_path;
        update_element_in_library(video_entry);
    }

    function update_element_with_tmdb(tmdb_result) {
        set_title_input(tmdb_result.title);
        //video_entry.year = tmdb_result.year;
        //video_entry.director = tmdb_result.director;
        //video_entry.countries = tmdb_result.countries;
        set_new_poster_url_or_path(tmdb_result.poster_path);
    }

    const [tmdb_searchfield, set_tmdb_searchfield] = useState("");
    const [tmdb_results, set_tmdb_results] = useState([]);

    async function get_tmdb_entries(search_string) {
        await invoke("search_tmdb_from_gui", {search_title: search_string})
          .then(res => {
            console.log(res);
            set_tmdb_results(res);
          })
          .catch(e => {
            console.log("Error: "+e);
          });
      }

    return(
        <div>
            <div style={{cursor: "pointer"}} onClick={() => {
                disable_view();
            }}>
                <i className="fa fa-angle-left" style={{fontSize: "48px", color: "white"}}></i>
            </div>
            Filepath: {video_entry.filepath}
            <div>Title: <input id="titleinput" type="text"
                               className="w-full px-3 py-2 border rounded-md border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
                               value={title_input}
                               onChange={(e) => {
                                    set_title_input(e.target.value);
                                    set_was_changed(true);
                               }}/></div>
            <div>Year: {video_entry.year}</div>
            <div>Director: {video_entry.director}</div>
            <div>Countries: <input id="titleinput" type="text"
                               className="w-full px-3 py-2 border rounded-md border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
                               value={countries_input.join(', ')}
                               onChange={(e) => {
                                    set_countries_input(e.target.value.split(/, |,/));
                                    set_was_changed(true);
                               }}/></div>
            <div>Watched: {video_entry.watched}</div>
            <div>Poster Path: {video_entry.poster_path}</div>

            <br/>
            {was_changed && <>
                <div style={{cursor: "pointer"}} onClick={() => {commit_changes();disable_view();}}>
                    Save Changes</div>
                <div style={{cursor: "pointer"}} onClick={() => {disable_view();}}>
                    Discard Changes</div>
            </>}

            <br/>
            Search Input: <input id="titleinput" type="text"
                                 className="w-full px-3 py-2 border rounded-md border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                 value={tmdb_searchfield}
                                 onChange={(e) => {
                                     set_tmdb_searchfield(e.target.value)
                                 }}/>
            <div style={{cursor: "pointer"}}
                 onClick={() => {
                    get_tmdb_entries(tmdb_searchfield);
                 }}>
                Search TMDB</div>
            <br/>
            {tmdb_results.map((result, index) => {
                return (

                    <div className={"tmdbresult"} key={index}
                         onClick={() => {
                             update_element_with_tmdb(result);
                             set_was_changed(true);
                         }}>
                        <div className={"tmdbresult-splitter"}>
                            <div className={"tmdbresult-img"}>
                                <img src={result.poster_path} alt={result.title}/>
                            </div>
                            <div className={"tmdbresult-info"}>
                                <div>{result.title}</div>
                                <div>{result.year}</div>
                            </div>
                        </div>
                    </div>
                )
            })}
        </div>
    )

}