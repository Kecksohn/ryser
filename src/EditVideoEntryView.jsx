import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

import editVideoStyles from "./EditVideoEntry.module.css";
import tmdbResultsStyles from "./TMDBResults.module.css";

export const EditVideoEntryView = ({disable_view, update_element_in_library, video_entry}) => {

    const [original_title_input, set_original_title_input] = useState(video_entry.original_title ? video_entry.original_title : "");
    const [english_title_input, set_english_title_input] = useState(video_entry.title ? video_entry.title : "");
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
        video_entry.original_title = original_title_input;
        video_entry.title = english_title_input;
        video_entry.year = year_input !== "" ? parseInt(year_input) : null;
        video_entry.director = director_input;
        video_entry.countries = countries_input;
        video_entry.poster_path = new_image_url_or_path;
        update_element_in_library(video_entry);
    }

    function update_element_with_tmdb(tmdb_result) {
        set_english_title_input(tmdb_result.title);
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
            { /* Back Button */}
            <div style={{cursor: "pointer"}} 
                onClick={() => { disable_view(); }}>
                <i className="fa fa-angle-left" style={{fontSize: "48px", color: "white"}}></i>
            </div>

            <div className={editVideoStyles.container}>
                { /* Cover */}
                <div className={editVideoStyles.containerBox} style={{textAlign: "right"}}>
                    <img style={{height: "500px", display: "block", marginLeft: "auto"}} src={new_image_url_or_path}/>
                </div>
                { /* Edit Info Box */}
                <div className={editVideoStyles.containerBox}>
                    
            Filepath: {video_entry.filepath}
                    <div>Original Title: <input id="og_title_input" type="text" className={editVideoStyles.inputField}
                               value={original_title_input}
                               onChange={(e) => {
                                   set_original_title_input(e.target.value);
                                   set_was_changed(true);
                               }}/></div>
                    <div>English  Title: <input id="en_title_input" type="text" className={editVideoStyles.inputField}
                               value={english_title_input}
                               onChange={(e) => {
                                    set_english_title_input(e.target.value);
                                    set_was_changed(true);
                               }}/></div>
            <div>Year: {video_entry.year}</div>
            <div>Director: {video_entry.director}</div>
                    <div>Countries: <input id="country_input" type="text" className={editVideoStyles.inputField}
                               value={countries_input.join(', ')}
                               onChange={(e) => {
                                    set_countries_input(e.target.value.split(/, |,/));
                                    set_was_changed(true);
                               }}/></div>
            <div>Watched: {video_entry.watched}</div>

            <br/>
            {was_changed && <>
                <div style={{cursor: "pointer"}} onClick={() => {commit_changes();disable_view();}}>
                    Save Changes</div>
                <div style={{cursor: "pointer"}} onClick={() => {disable_view();}}>
                    Discard Changes</div>
            </>}

                    
                 </div>
            </div>

            <br/>
            Search Input: <input id="searchinput" type="text"
                                    className={editVideoStyles.inputField}
                                 value={tmdb_searchfield}
                                 onChange={(e) => {
                                     set_tmdb_searchfield(e.target.value)
                                 }}/>
            <span style={{cursor: "pointer"}}
                 onClick={() => {
                    get_tmdb_entries(tmdb_searchfield);
                 }}> Search TMDB</span>
            <br/>
            {tmdb_results.map((result, index) => {
                return (

                    <div className={tmdbResultsStyles.tmdbresult} key={index}
                         onClick={() => {
                             update_element_with_tmdb(result);
                             set_was_changed(true);
                         }}>
                        <div className={tmdbResultsStyles.tmdbresultSplitter}>
                            <div className={tmdbResultsStyles.tmdbresultImg}>
                                <img src={result.poster_path} alt={result.title}/>
                            </div>
                            <div className={tmdbResultsStyles.tmdbresultInfo}>
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