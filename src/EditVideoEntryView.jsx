import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export const EditVideoEntryView = ({disable_view, video_entry}) => {


    return(
        <div onClick={() => {disable_view();}} style={{cursor: "pointer"}}>
            Filepath: {video_entry.filepath}
            <div>Name: {video_entry.title}</div>
            <div>Year: {video_entry.year}</div>
            <div>Director: {video_entry.director}</div>
            <div>Countries: {video_entry.countries}</div>
            <div>Poster Path: {video_entry.poster_path}</div>
        </div>
    )

}